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

    pub fn is_company_admin(self) -> bool {
        matches!(
            self,
            TenantScope::Company {
                role: CompanyRole::CompanyAdmin,
                ..
            }
        )
    }

    pub fn session_user_id(self) -> Uuid {
        match self {
            TenantScope::Personal { user_id } => user_id,
            TenantScope::Company { user_id, .. } => user_id,
        }
    }

    /// `(company_id, owner_user_id, is_company_admin, session_user_id)` for project access SQL.
    pub fn project_access_binds(self) -> (Option<Uuid>, Option<Uuid>, bool, Uuid) {
        (
            self.company_id_or_null(),
            self.owner_user_id_or_null(),
            self.is_company_admin(),
            self.session_user_id(),
        )
    }
}
