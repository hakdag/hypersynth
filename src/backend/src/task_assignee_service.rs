use axum::http::StatusCode;
use axum::Json;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::types::{ApiErrorBody, TenantScope};

pub async fn resolve_assignee(
    conn: &mut PgConnection,
    scope: TenantScope,
    project_id: Uuid,
    unassigned: bool,
    assignee_user_id: Option<Uuid>,
) -> Result<Option<Uuid>, (StatusCode, Json<ApiErrorBody>)> {
    if unassigned {
        return Ok(None);
    }

    let resolved = assignee_user_id.unwrap_or(scope.session_user_id());
    match scope {
        TenantScope::Personal { user_id } => {
            if resolved != user_id {
                return Err(bad_request(
                    "That user cannot be assigned in this workspace.",
                ));
            }
            Ok(Some(resolved))
        }
        TenantScope::Company { company_id, .. } => {
            let in_company: bool = sqlx::query_scalar(
                r#"
                SELECT EXISTS(
                    SELECT 1 FROM company_users cu
                    WHERE cu.company_id = $1 AND cu.user_id = $2
                )
                "#,
            )
            .bind(company_id)
            .bind(resolved)
            .fetch_one(&mut *conn)
            .await
            .map_err(|_| internal_error())?;

            if !in_company {
                return Err(bad_request(
                    "That user cannot be assigned in this workspace.",
                ));
            }

            let in_project: bool = sqlx::query_scalar(
                r#"
                SELECT EXISTS(
                    SELECT 1 FROM project_memberships pm
                    WHERE pm.project_id = $1 AND pm.user_id = $2
                )
                "#,
            )
            .bind(project_id)
            .bind(resolved)
            .fetch_one(&mut *conn)
            .await
            .map_err(|_| internal_error())?;

            if !in_project {
                return Err(bad_request("Assignee must be a member of this project."));
            }

            Ok(Some(resolved))
        }
    }
}

fn bad_request(message: impl Into<String>) -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ApiErrorBody {
            message: message.into(),
            ..Default::default()
        }),
    )
}

fn internal_error() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiErrorBody {
            message: "Something went wrong. Please try again.".into(),
            ..Default::default()
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::resolve_assignee;
    use crate::types::{CompanyRole, TenantScope};
    use axum::http::StatusCode;
    use sqlx::PgPool;
    use uuid::Uuid;

    async fn connect_pool() -> Option<PgPool> {
        let database_url = std::env::var("DATABASE_URL").ok()?;
        PgPool::connect(&database_url).await.ok()
    }

    async fn seed_company_scope(
        pool: &PgPool,
    ) -> Result<(Uuid, Uuid, Uuid, Uuid, Uuid), Box<dyn std::error::Error>> {
        let mut tx = pool.begin().await?;
        let company_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();
        let feature_id = Uuid::new_v4();
        let member_user_id = Uuid::new_v4();
        let outsider_user_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO companies (id, name, company_email, country, timezone, status)
            VALUES ($1, 'SF29 Co', $2, 'TR', 'UTC', 'active')
            "#,
        )
        .bind(company_id)
        .bind(format!("sf29-{}@example.test", Uuid::new_v4()))
        .execute(&mut *tx)
        .await?;

        for (user_id, email) in [
            (
                member_user_id,
                format!("member-{}@example.test", Uuid::new_v4()),
            ),
            (
                outsider_user_id,
                format!("outsider-{}@example.test", Uuid::new_v4()),
            ),
        ] {
            sqlx::query(
                r#"
                INSERT INTO users (id, fullname, email, password_hash, account_type, role)
                VALUES ($1, 'Test User', $2, 'unused', 'company', 'contributor')
                "#,
            )
            .bind(user_id)
            .bind(email)
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query(
            r#"
            INSERT INTO company_users (company_id, user_id) VALUES ($1, $2), ($1, $3)
            "#,
        )
        .bind(company_id)
        .bind(member_user_id)
        .bind(outsider_user_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO projects (id, owner_user_id, company_id, created_by_user_id, name, status)
            VALUES ($1, NULL, $2, $3, 'SF29 Project', 'Pending')
            "#,
        )
        .bind(project_id)
        .bind(company_id)
        .bind(member_user_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO features (id, project_id, title, status)
            VALUES ($1, $2, 'SF29 Feature', 'Pending')
            "#,
        )
        .bind(feature_id)
        .bind(project_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO project_memberships (project_id, user_id, role)
            VALUES ($1, $2, 'contributor')
            "#,
        )
        .bind(project_id)
        .bind(member_user_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok((
            company_id,
            project_id,
            feature_id,
            member_user_id,
            outsider_user_id,
        ))
    }

    #[tokio::test]
    async fn company_scope_requires_project_member_assignee() -> Result<(), Box<dyn std::error::Error>>
    {
        let Some(pool) = connect_pool().await else {
            return Ok(());
        };
        sqlx::migrate!("./migrations").run(&pool).await?;
        let (company_id, project_id, _, member_user_id, outsider_user_id) =
            seed_company_scope(&pool).await?;

        let mut conn = pool.acquire().await?;
        let scope = TenantScope::Company {
            company_id,
            user_id: member_user_id,
            role: CompanyRole::Contributor,
        };

        let ok = resolve_assignee(
            &mut conn,
            scope,
            project_id,
            false,
            Some(member_user_id),
        )
        .await
        .expect("project member assignee should be accepted");
        assert_eq!(ok, Some(member_user_id));

        let err = resolve_assignee(
            &mut conn,
            scope,
            project_id,
            false,
            Some(outsider_user_id),
        )
        .await
        .expect_err("outsider should be rejected");
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
        assert_eq!(err.1.message, "Assignee must be a member of this project.");
        Ok(())
    }

    #[tokio::test]
    async fn personal_scope_rejects_foreign_assignee() -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = connect_pool().await else {
            return Ok(());
        };
        sqlx::migrate!("./migrations").run(&pool).await?;

        let owner_user_id = Uuid::new_v4();
        let foreign_user_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();
        let feature_id = Uuid::new_v4();
        let mut tx = pool.begin().await?;

        for (id, email) in [
            (owner_user_id, format!("owner-{}@example.test", Uuid::new_v4())),
            (foreign_user_id, format!("foreign-{}@example.test", Uuid::new_v4())),
        ] {
            sqlx::query(
                r#"
                INSERT INTO users (id, fullname, email, password_hash, account_type, role)
                VALUES ($1, 'Personal User', $2, 'unused', 'personal', NULL)
                "#,
            )
            .bind(id)
            .bind(email)
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query(
            r#"
            INSERT INTO projects (id, owner_user_id, company_id, created_by_user_id, name, status)
            VALUES ($1, $2, NULL, $2, 'Personal Project', 'Pending')
            "#,
        )
        .bind(project_id)
        .bind(owner_user_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO features (id, project_id, title, status)
            VALUES ($1, $2, 'Personal Feature', 'Pending')
            "#,
        )
        .bind(feature_id)
        .bind(project_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        let mut conn = pool.acquire().await?;
        let scope = TenantScope::Personal {
            user_id: owner_user_id,
        };

        let ok = resolve_assignee(&mut conn, scope, project_id, false, Some(owner_user_id))
            .await
            .expect("personal owner assignee should be accepted");
        assert_eq!(ok, Some(owner_user_id));

        let err = resolve_assignee(&mut conn, scope, project_id, false, Some(foreign_user_id))
            .await
            .expect_err("foreign personal assignee should be rejected");
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
        assert_eq!(err.1.message, "That user cannot be assigned in this workspace.");
        Ok(())
    }
}
