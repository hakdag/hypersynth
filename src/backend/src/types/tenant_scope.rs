use uuid::Uuid;

use crate::types::CompanyRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TenantScope {
    Personal {
        user_id: Uuid,
    },
    Company {
        company_id: Uuid,
        user_id: Uuid,
        role: CompanyRole,
    },
}

impl TenantScope {
    pub fn owner_user_id_or_null(self) -> Option<Uuid> {
        match self {
            TenantScope::Personal { user_id } => Some(user_id),
            TenantScope::Company { .. } => None,
        }
    }

    pub fn company_id_or_null(self) -> Option<Uuid> {
        match self {
            TenantScope::Personal { .. } => None,
            TenantScope::Company { company_id, .. } => Some(company_id),
        }
    }
}
