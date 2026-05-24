/// Debezium-envelope `op` codes recorded in `audit_row_changes`.
#[derive(Debug, Clone, Copy)]
pub enum AuditOp {
    Create,
    Update,
    Delete,
}

impl AuditOp {
    pub fn as_str(self) -> &'static str {
        match self {
            AuditOp::Create => "c",
            AuditOp::Update => "u",
            AuditOp::Delete => "d",
        }
    }

    pub fn from_db_value(value: &str) -> Option<Self> {
        match value {
            "c" => Some(AuditOp::Create),
            "u" => Some(AuditOp::Update),
            "d" => Some(AuditOp::Delete),
            _ => None,
        }
    }
}
