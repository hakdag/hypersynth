use serde::Serialize;
use uuid::Uuid;

/// Actor portion of an audit envelope. Serialised into the `actor`
/// JSONB column of both `audit_row_changes` and `audit_events`, and
/// pushed into the transaction-local GUC `app.actor` by the audit
/// middleware so the row-change trigger can stamp it onto every
/// captured row.
///
/// For unauthenticated requests, `AuditActor::anonymous()` is used and
/// every field is `None` / `false`.
#[derive(Debug, Serialize)]
pub struct AuditActor {
    pub system_admin: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_id: Option<Uuid>,
}

impl AuditActor {
    pub fn anonymous() -> Self {
        Self {
            system_admin: false,
            user_id: None,
            email: None,
            account_type: None,
            company_id: None,
        }
    }
}
