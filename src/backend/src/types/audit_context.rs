use std::net::IpAddr;
use std::sync::Arc;

use serde_json::Value;
use uuid::Uuid;

/// Snapshot of per-request audit context computed once by the audit
/// middleware and shared, via request `Extensions`, with any handler
/// that needs to record a non-data audit event through
/// `AuditEventsService`.
///
/// The same values are also serialised into Postgres GUCs (`app.actor`,
/// `app.request_id`, `app.ip_address`, `app.user_agent`) on the request
/// transaction so that the row-change triggers see them. This struct is
/// the application-side mirror so that fire-and-forget audit event
/// writes (which run on a separate pool connection, outside the request
/// transaction) carry the same context.
#[derive(Debug)]
pub struct AuditContext {
    pub actor: Value,
    pub request_id: Uuid,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
}

/// Shared handle to an `AuditContext` placed into request extensions
/// by the audit middleware.
pub type SharedAuditContext = Arc<AuditContext>;
