use shaperail_core::ShaperailError;
use shaperail_runtime::handlers::controller::{Context, ControllerResult};

/// Called before create — auto-fill `created_by` from the authenticated user's token.
pub async fn set_created_by(ctx: &mut Context) -> ControllerResult {
    match &ctx.user {
        Some(user) => {
            ctx.input["created_by"] = serde_json::json!(user.user_id);
            Ok(())
        }
        None => Err(ShaperailError::Auth(
            "Authentication required to create posts".into(),
        )),
    }
}
