/// Lifecycle events recorded into `project_api_key_audit`.
///
/// Variants map directly to the `event_type` CHECK constraint values.
#[derive(Debug, Clone, Copy)]
pub enum ApiKeyAuditEvent {
    Created,
    Replaced,
    Cleared,
    /// Emitted from the future runtime decryption path; ships unused in SF-17.
    #[allow(dead_code)]
    RuntimeUse,
}

impl ApiKeyAuditEvent {
    pub fn as_str(self) -> &'static str {
        match self {
            ApiKeyAuditEvent::Created => "created",
            ApiKeyAuditEvent::Replaced => "replaced",
            ApiKeyAuditEvent::Cleared => "cleared",
            ApiKeyAuditEvent::RuntimeUse => "runtime_use",
        }
    }
}
