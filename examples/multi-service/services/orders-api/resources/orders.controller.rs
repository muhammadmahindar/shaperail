use shaperail_core::{FieldError, ShaperailError};
use shaperail_runtime::handlers::controller::{Context, ControllerResult};
use tracing::warn;

/// Before-create: validate user exists, enforce positive total, set defaults.
///
/// Cross-service validation: queries the users table to confirm the referenced
/// user_id exists. In a shared-database workspace this is a direct DB query;
/// in a split-database topology, replace with a typed HTTP client call to the
/// users-api service.
pub async fn validate_order(ctx: &mut Context) -> ControllerResult {
    // --- Validate user_id exists (cross-service) ---
    let user_id = ctx
        .input
        .get("user_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ShaperailError::Validation(vec![FieldError {
                field: "user_id".to_string(),
                message: "user_id is required".to_string(),
                code: "required".to_string(),
            }])
        })?;

    let user_exists: (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1::uuid)")
            .bind(user_id)
            .fetch_one(&ctx.pool)
            .await
            .map_err(|e| ShaperailError::Internal(e.to_string()))?;

    if !user_exists.0 {
        return Err(ShaperailError::Validation(vec![FieldError {
            field: "user_id".to_string(),
            message: format!("User '{user_id}' does not exist"),
            code: "invalid_reference".to_string(),
        }]));
    }

    // --- Validate total is positive ---
    let total = ctx
        .input
        .get("total")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| {
            ShaperailError::Validation(vec![FieldError {
                field: "total".to_string(),
                message: "total is required and must be a number".to_string(),
                code: "required".to_string(),
            }])
        })?;

    if total <= 0.0 {
        return Err(ShaperailError::Validation(vec![FieldError {
            field: "total".to_string(),
            message: "Order total must be positive".to_string(),
            code: "invalid_value".to_string(),
        }]));
    }

    // --- Override status to "pending" regardless of client input ---
    ctx.input
        .insert("status".to_string(), serde_json::json!("pending"));

    // --- Generate order_number: ORD-{epoch_secs}-{random_suffix} ---
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let random_suffix: u32 = rand::random::<u32>() % 100_000;
    let order_number = format!("ORD-{now}-{random_suffix:05}");
    ctx.input
        .insert("order_number".to_string(), serde_json::json!(order_number));

    // --- Auto-fill created_by from JWT ---
    if let Some(user) = &ctx.user {
        ctx.input
            .insert("created_by".to_string(), serde_json::json!(user.id));
    }

    Ok(())
}

/// Before-update: enforce the order status state machine.
///
/// Valid transitions:
///   pending  -> paid
///   paid     -> shipped      (admin only)
///   paid     -> cancelled    (refund flagged)
///   shipped  -> delivered    (admin only)
///   pending  -> cancelled
///
/// Cancelled and delivered orders cannot be modified.
pub async fn enforce_order_status(ctx: &mut Context) -> ControllerResult {
    let caller = ctx.user.as_ref().ok_or(ShaperailError::Unauthorized)?;

    let new_status = match ctx.input.get("status").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return Ok(()), // No status change requested — nothing to enforce.
    };

    // Fetch the current order record from the DB to get its current status.
    let current_status = ctx
        .data
        .as_ref()
        .and_then(|d| d.get("status"))
        .and_then(|v| v.as_str())
        .unwrap_or("pending");

    // Cancelled and delivered orders are immutable.
    if current_status == "cancelled" || current_status == "delivered" {
        return Err(ShaperailError::Validation(vec![FieldError {
            field: "status".to_string(),
            message: format!("Cannot modify an order with status '{current_status}'"),
            code: "immutable_order".to_string(),
        }]));
    }

    // Define allowed transitions.
    let allowed = match current_status {
        "pending" => &["paid", "cancelled"][..],
        "paid" => &["shipped", "cancelled"][..],
        "shipped" => &["delivered"][..],
        _ => &[][..],
    };

    if !allowed.contains(&new_status.as_str()) {
        return Err(ShaperailError::Validation(vec![FieldError {
            field: "status".to_string(),
            message: format!(
                "Invalid transition: '{current_status}' -> '{new_status}'. \
                 Allowed: {allowed:?}"
            ),
            code: "invalid_transition".to_string(),
        }]));
    }

    // Only admins can move to "shipped" or "delivered".
    if (new_status == "shipped" || new_status == "delivered") && caller.role != "admin" {
        return Err(ShaperailError::Validation(vec![FieldError {
            field: "status".to_string(),
            message: format!(
                "Only admins can transition orders to '{new_status}'"
            ),
            code: "insufficient_permissions".to_string(),
        }]));
    }

    // When cancelling a paid order, flag that a refund is needed.
    if new_status == "cancelled" && current_status == "paid" {
        let total = ctx
            .data
            .as_ref()
            .and_then(|d| d.get("total"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        if total > 0.0 {
            warn!(
                order_id = ctx.data.as_ref().and_then(|d| d.get("id")).and_then(|v| v.as_str()).unwrap_or("unknown"),
                total = total,
                "Refund needed for cancelled paid order"
            );
            ctx.input
                .insert("refund_required".to_string(), serde_json::json!(true));
            ctx.input
                .insert("refund_amount".to_string(), serde_json::json!(total));
        }
    }

    Ok(())
}
