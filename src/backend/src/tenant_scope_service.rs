use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::authorization;
use crate::types::{AccountType, ApiErrorBody, CompanyRole, SessionUser, TenantScope};

pub struct TenantScopeService;

impl TenantScopeService {
    pub fn from_session(
        user: &SessionUser,
    ) -> Result<TenantScope, (StatusCode, Json<ApiErrorBody>)> {
        match user.account_type {
            AccountType::Personal => Ok(TenantScope::Personal { user_id: user.id }),
            AccountType::SystemAdmin => Err(authorization::forbidden(
                "This action is not available to system administrators.",
            )),
            AccountType::Company => {
                let company_id = user.company_id.ok_or_else(|| {
                    authorization::forbidden("Company membership is required for this action.")
                })?;
                let role = user.role.ok_or_else(|| {
                    authorization::forbidden("Company role is required for this action.")
                })?;
                Ok(TenantScope::Company {
                    company_id,
                    user_id: user.id,
                    role,
                })
            }
        }
    }

    #[allow(dead_code)]
    pub async fn require_project_access(
        pool: &sqlx::PgPool,
        scope: TenantScope,
        project_id: Uuid,
    ) -> Result<(), (StatusCode, Json<ApiErrorBody>)> {
        let found: Option<(Uuid,)> = match scope {
            TenantScope::Personal { user_id } => {
                sqlx::query_as(
                    r#"
                    SELECT id
                    FROM projects
                    WHERE id = $1
                      AND owner_user_id = $2
                      AND company_id IS NULL
                    "#,
                )
                .bind(project_id)
                .bind(user_id)
                .fetch_optional(pool)
                .await
            }
            TenantScope::Company {
                company_id,
                user_id,
                role,
            } => {
                let is_admin = role == CompanyRole::CompanyAdmin;
                sqlx::query_as(
                    r#"
                    SELECT id
                    FROM projects
                    WHERE id = $1
                      AND company_id = $2
                      AND (
                        $3::boolean
                        OR EXISTS (
                            SELECT 1 FROM project_memberships pm
                            WHERE pm.project_id = projects.id AND pm.user_id = $4
                        )
                    )
                    "#,
                )
                .bind(project_id)
                .bind(company_id)
                .bind(is_admin)
                .bind(user_id)
                .fetch_optional(pool)
                .await
            }
        }
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorBody {
                    message: "Something went wrong. Please try again.".into(),
                    ..Default::default()
                }),
            )
        })?;

        if found.is_some() {
            Ok(())
        } else {
            Err((
                StatusCode::NOT_FOUND,
                Json(ApiErrorBody {
                    message: "Project not found.".into(),
                    ..Default::default()
                }),
            ))
        }
    }
}
