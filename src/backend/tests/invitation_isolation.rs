use sqlx::PgPool;
use uuid::Uuid;

async fn connect_pool() -> Option<PgPool> {
    let database_url = std::env::var("DATABASE_URL").ok()?;
    PgPool::connect(&database_url).await.ok()
}

/// SF-15 / SF-14: pending invitations are unique per (company, lower(email)); cancel is scoped by company.
#[tokio::test]
async fn invitations_pending_unique_and_cancel_scoped_by_company(
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };

    sqlx::migrate!("./migrations").run(&pool).await?;

    let mut tx = pool.begin().await?;

    let company_id = Uuid::new_v4();
    let other_company_id = Uuid::new_v4();
    let inviter_id = Uuid::new_v4();
    let invitation_id = Uuid::new_v4();
    let email = format!("invitee-{}@example.test", Uuid::new_v4());

    sqlx::query(
        r#"
        INSERT INTO companies (id, name, company_email, country, timezone, status)
        VALUES ($1, 'Co A', $2, 'TR', 'UTC', 'active')
        "#,
    )
    .bind(company_id)
    .bind(format!("co-a-{}@example.test", Uuid::new_v4()))
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO companies (id, name, company_email, country, timezone, status)
        VALUES ($1, 'Co B', $2, 'TR', 'UTC', 'active')
        "#,
    )
    .bind(other_company_id)
    .bind(format!("co-b-{}@example.test", Uuid::new_v4()))
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO users (id, fullname, email, password_hash, account_type, role, status)
        VALUES ($1, 'Admin', $2, 'hash', 'company', 'company_admin', 'active')
        "#,
    )
    .bind(inviter_id)
    .bind(format!("admin-{}@example.test", Uuid::new_v4()))
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO company_users (company_id, user_id) VALUES ($1, $2)",
    )
    .bind(company_id)
    .bind(inviter_id)
    .execute(&mut *tx)
    .await?;

    let token_hash = "a".repeat(64);
    let expires = chrono::Utc::now() + chrono::Duration::hours(24);

    sqlx::query(
        r#"
        INSERT INTO invitations (
            id, invitation_token_hash, company_id, project_id,
            invited_email, invited_role, invited_by_user_id, status, expires_at
        )
        VALUES ($1, $2, $3, NULL, $4, 'contributor', $5, 'pending', $6)
        "#,
    )
    .bind(invitation_id)
    .bind(&token_hash)
    .bind(company_id)
    .bind(&email)
    .bind(inviter_id)
    .bind(expires)
    .execute(&mut *tx)
    .await?;

    let dup = sqlx::query(
        r#"
        INSERT INTO invitations (
            invitation_token_hash, company_id, project_id,
            invited_email, invited_role, invited_by_user_id, status, expires_at
        )
        VALUES ($1, $2, NULL, $3, 'viewer', $4, 'pending', $5)
        "#,
    )
    .bind("b".repeat(64))
    .bind(company_id)
    .bind(&email)
    .bind(inviter_id)
    .bind(expires)
    .execute(&mut *tx)
    .await;

    assert!(dup.is_err(), "expected duplicate pending invitation to violate unique index");

    let rows = sqlx::query(
        r#"
        UPDATE invitations
        SET status = 'cancelled'
        WHERE id = $1 AND company_id = $2 AND status = 'pending'
        RETURNING id
        "#,
    )
    .bind(invitation_id)
    .bind(other_company_id)
    .fetch_all(&mut *tx)
    .await?;

    assert!(
        rows.is_empty(),
        "cancel must not affect invitation when company_id does not match (SF-14)"
    );

    let rows = sqlx::query(
        r#"
        UPDATE invitations
        SET status = 'cancelled'
        WHERE id = $1 AND company_id = $2 AND status = 'pending'
        RETURNING id
        "#,
    )
    .bind(invitation_id)
    .bind(company_id)
    .fetch_all(&mut *tx)
    .await?;

    assert_eq!(rows.len(), 1, "cancel with matching company_id should succeed");

    tx.rollback().await?;
    Ok(())
}
