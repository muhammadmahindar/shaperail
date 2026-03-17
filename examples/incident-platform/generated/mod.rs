#![allow(dead_code)]

pub mod alerts;
pub mod attachments;
pub mod incidents;
pub mod services;

pub fn build_store_registry(pool: sqlx::PgPool) -> shaperail_runtime::db::StoreRegistry {
    let mut stores: std::collections::HashMap<
        String,
        std::sync::Arc<dyn shaperail_runtime::db::ResourceStore>,
    > = std::collections::HashMap::new();
    stores.insert("alerts".to_string(), std::sync::Arc::new(alerts::AlertsStore::new(pool.clone())));
    stores.insert("attachments".to_string(), std::sync::Arc::new(attachments::AttachmentsStore::new(pool.clone())));
    stores.insert("incidents".to_string(), std::sync::Arc::new(incidents::IncidentsStore::new(pool.clone())));
    stores.insert("services".to_string(), std::sync::Arc::new(services::ServicesStore::new(pool.clone())));
    std::sync::Arc::new(stores)
}

/// Returns an empty controller map. Register custom controller functions here
/// or populate from `resources/<name>.controller.rs` files.
pub fn build_controller_map() -> shaperail_runtime::handlers::controller::ControllerMap {
    shaperail_runtime::handlers::controller::ControllerMap::new()
}


/// Input fields for the alerts create endpoint.
/// Auto-generated from the resource schema — do not edit.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AlertsCreateInput {
    pub org_id: uuid::Uuid,
    pub service_id: uuid::Uuid,
    pub incident_id: Option<uuid::Uuid>,
    pub external_id: String,
    pub source: String,
    pub severity: Option<String>,
    pub fingerprint: String,
    pub summary: String,
    pub payload: serde_json::Value,
}

/// Input fields for the alerts update endpoint.
/// Auto-generated from the resource schema — do not edit.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AlertsUpdateInput {
    pub incident_id: Option<uuid::Uuid>,
    pub status: Option<String>,
}

/// Controller trait for the alerts resource.
/// Implement this trait in `controllers/alerts.controller.rs`.
/// The compiler will enforce correct signatures — no guessing needed.
#[shaperail_runtime::db::async_trait]
pub trait AlertsController {
    /// Before-hook for the create endpoint. Called before the DB operation.
    async fn ingest_alert(ctx: &shaperail_runtime::handlers::ControllerContext, input: &AlertsCreateInput) -> Result<(), shaperail_core::ShaperailError>;

    /// Before-hook for the update endpoint. Called before the DB operation.
    async fn reconcile_alert_link(ctx: &shaperail_runtime::handlers::ControllerContext, input: &AlertsUpdateInput) -> Result<(), shaperail_core::ShaperailError>;
}

/// Input fields for the incidents create endpoint.
/// Auto-generated from the resource schema — do not edit.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IncidentsCreateInput {
    pub service_id: uuid::Uuid,
    pub title: String,
    pub slug: String,
    pub severity: Option<String>,
    pub summary: String,
    pub commander_id: Option<uuid::Uuid>,
    pub room_key: String,
    pub created_by: uuid::Uuid,
}

/// Input fields for the incidents update endpoint.
/// Auto-generated from the resource schema — do not edit.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IncidentsUpdateInput {
    pub status: Option<String>,
    pub summary: String,
    pub commander_id: Option<uuid::Uuid>,
}

/// Controller trait for the incidents resource.
/// Implement this trait in `controllers/incidents.controller.rs`.
/// The compiler will enforce correct signatures — no guessing needed.
#[shaperail_runtime::db::async_trait]
pub trait IncidentsController {
    /// Before-hook for the create endpoint. Called before the DB operation.
    async fn open_incident(ctx: &shaperail_runtime::handlers::ControllerContext, input: &IncidentsCreateInput) -> Result<(), shaperail_core::ShaperailError>;

    /// Before-hook for the update endpoint. Called before the DB operation.
    async fn enforce_incident_update(ctx: &shaperail_runtime::handlers::ControllerContext, input: &IncidentsUpdateInput) -> Result<(), shaperail_core::ShaperailError>;

    /// After-hook for the update endpoint. Called after the DB operation.
    async fn write_incident_audit(ctx: &shaperail_runtime::handlers::ControllerContext, result: &serde_json::Value) -> Result<serde_json::Value, shaperail_core::ShaperailError>;
}

/// Input fields for the services create endpoint.
/// Auto-generated from the resource schema — do not edit.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServicesCreateInput {
    pub name: String,
    pub slug: String,
    pub tier: Option<String>,
    pub status: Option<String>,
    pub owner_team: String,
    pub runbook_url: Option<String>,
    pub created_by: uuid::Uuid,
}

/// Controller trait for the services resource.
/// Implement this trait in `controllers/services.controller.rs`.
/// The compiler will enforce correct signatures — no guessing needed.
#[shaperail_runtime::db::async_trait]
pub trait ServicesController {
    /// Before-hook for the create endpoint. Called before the DB operation.
    async fn prepare_service(ctx: &shaperail_runtime::handlers::ControllerContext, input: &ServicesCreateInput) -> Result<(), shaperail_core::ShaperailError>;
}

