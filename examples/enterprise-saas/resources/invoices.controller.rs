use shaperail_core::{FieldError, ShaperailError};
use shaperail_runtime::handlers::controller::{Context, ControllerResult};

/// Called before create — auto-generate invoice_number, validate customer exists
/// and is active, default status to "draft", and auto-fill created_by.
pub async fn prepare_invoice(ctx: &mut Context) -> ControllerResult {
    // Auto-fill created_by from JWT
    let user = ctx.user.as_ref().ok_or_else(|| {
        ShaperailError::Auth("Authentication required to create invoices".into())
    })?;
    ctx.input["created_by"] = serde_json::json!(user.id);

    // Auto-fill org_id from tenant context
    if let Some(ref tenant_id) = ctx.tenant_id {
        ctx.input["org_id"] = serde_json::json!(tenant_id);
    }

    // Validate customer exists and is active
    let customer_id = ctx
        .input
        .get("customer_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ShaperailError::Validation(vec![FieldError {
                field: "customer_id".into(),
                message: "customer_id is required".into(),
                code: "required".into(),
            }])
        })?
        .to_string();

    let customer_status: Option<String> =
        sqlx::query_scalar("SELECT status FROM customers WHERE id = $1 AND deleted_at IS NULL")
            .bind(&customer_id)
            .fetch_optional(&ctx.pool)
            .await
            .map_err(|e| ShaperailError::Internal(format!("DB error fetching customer: {e}")))?;

    match customer_status.as_deref() {
        None => {
            return Err(ShaperailError::Validation(vec![FieldError {
                field: "customer_id".into(),
                message: format!("Customer '{customer_id}' not found"),
                code: "not_found".into(),
            }]));
        }
        Some("suspended") | Some("closed") => {
            return Err(ShaperailError::Validation(vec![FieldError {
                field: "customer_id".into(),
                message: format!(
                    "Customer '{customer_id}' has status '{}' and cannot receive new invoices",
                    customer_status.unwrap()
                ),
                code: "inactive_customer".into(),
            }]));
        }
        Some("active") => {} // ok
        Some(other) => {
            return Err(ShaperailError::Internal(format!(
                "Unknown customer status: {other}"
            )));
        }
    }

    // Auto-generate invoice_number: INV-{YYYYMMDD}-{sequential}
    let today = chrono::Utc::now().format("%Y%m%d").to_string();
    let prefix = format!("INV-{today}-");

    let last_seq: Option<String> = sqlx::query_scalar(
        "SELECT invoice_number FROM invoices \
         WHERE invoice_number LIKE $1 \
         ORDER BY invoice_number DESC LIMIT 1",
    )
    .bind(format!("{prefix}%"))
    .fetch_optional(&ctx.pool)
    .await
    .map_err(|e| ShaperailError::Internal(format!("DB error generating invoice number: {e}")))?;

    let next_seq = match last_seq {
        Some(last) => {
            let seq_str = last.strip_prefix(&prefix).unwrap_or("0000");
            let seq: u32 = seq_str.parse().unwrap_or(0);
            seq + 1
        }
        None => 1,
    };

    let invoice_number = format!("{prefix}{next_seq:04}");
    ctx.input["invoice_number"] = serde_json::json!(invoice_number);

    // Default status to "draft"
    ctx.input["status"] = serde_json::json!("draft");

    Ok(())
}

/// Called before update — enforce the invoice state machine:
///   draft -> pending -> sent -> paid
///   draft -> void
///   sent -> overdue
/// Only finance role can transition draft -> sent.
/// Only admin can void.
/// Cannot edit paid or voided invoices.
/// When marking paid, set paid_at timestamp.
pub async fn enforce_invoice_workflow(ctx: &mut Context) -> ControllerResult {
    let resource_id = ctx
        .input
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Fetch current invoice state
    let row = sqlx::query_as::<_, (String,)>("SELECT status FROM invoices WHERE id = $1")
        .bind(resource_id)
        .fetch_one(&ctx.pool)
        .await
        .map_err(|e| ShaperailError::Internal(format!("DB error fetching invoice: {e}")))?;

    let current_status = &row.0;

    // Block edits to paid/void invoices entirely
    if current_status == "paid" || current_status == "void" {
        return Err(ShaperailError::Validation(vec![FieldError {
            field: "status".into(),
            message: format!("Cannot modify an invoice with status '{current_status}'"),
            code: "immutable_status".into(),
        }]));
    }

    let new_status = match ctx.input.get("status").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return Ok(()), // not changing status, allow other field edits
    };

    // Validate state transition
    let valid_transitions: &[(&str, &str)] = &[
        ("draft", "pending"),
        ("pending", "sent"),
        ("sent", "paid"),
        ("draft", "void"),
        ("sent", "overdue"),
    ];

    let is_valid = valid_transitions
        .iter()
        .any(|(from, to)| from == current_status && *to == new_status);

    if !is_valid {
        return Err(ShaperailError::Validation(vec![FieldError {
            field: "status".into(),
            message: format!(
                "Invalid status transition: '{current_status}' -> '{new_status}'"
            ),
            code: "invalid_transition".into(),
        }]));
    }

    // Role-based transition guards
    let user_role = ctx
        .user
        .as_ref()
        .map(|u| u.role.as_str())
        .unwrap_or("unknown");

    // Only finance role can send invoices (draft->sent via pending->sent)
    if new_status == "sent" && user_role != "finance" && user_role != "admin" {
        return Err(ShaperailError::Validation(vec![FieldError {
            field: "status".into(),
            message: "Only finance or admin roles can send invoices".into(),
            code: "insufficient_role".into(),
        }]));
    }

    // Only admin can void invoices
    if new_status == "void" && user_role != "admin" {
        return Err(ShaperailError::Validation(vec![FieldError {
            field: "status".into(),
            message: "Only admin role can void invoices".into(),
            code: "insufficient_role".into(),
        }]));
    }

    // When marking as sent, set sent_at
    if new_status == "sent" {
        ctx.input["sent_at"] = serde_json::json!(chrono::Utc::now().to_rfc3339());
    }

    // When marking as paid, set paid_at
    if new_status == "paid" {
        ctx.input["paid_at"] = serde_json::json!(chrono::Utc::now().to_rfc3339());
    }

    Ok(())
}

/// Called after update — write an audit trail entry with before/after snapshot.
pub async fn audit_invoice_change(ctx: &mut Context) -> ControllerResult {
    let resource_id = ctx
        .input
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let user_id = ctx
        .user
        .as_ref()
        .map(|u| u.id.to_string())
        .unwrap_or_default();

    let ip_address = ctx
        .headers
        .get("x-forwarded-for")
        .or_else(|| ctx.headers.get("x-real-ip"))
        .cloned()
        .unwrap_or_else(|| "unknown".into());

    // The before data is the input fields (what was changed).
    // The after data is the full record returned from the DB operation.
    let before_data = serde_json::json!(ctx.input);
    let after_data = ctx.data.clone().unwrap_or(serde_json::json!(null));

    sqlx::query(
        "INSERT INTO audit_logs (id, user_id, resource_type, resource_id, action, before_data, after_data, ip_address, created_at) \
         VALUES (gen_random_uuid(), $1, 'invoices', $2, 'update', $3, $4, $5, NOW())",
    )
    .bind(&user_id)
    .bind(resource_id)
    .bind(&before_data)
    .bind(&after_data)
    .bind(&ip_address)
    .execute(&ctx.pool)
    .await
    .map_err(|e| ShaperailError::Internal(format!("DB error writing audit log: {e}")))?;

    Ok(())
}
