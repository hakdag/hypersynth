use chrono::{Duration as ChronoDuration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

async fn connect_pool() -> Option<PgPool> {
    let database_url = std::env::var("DATABASE_URL").ok()?;
    PgPool::connect(&database_url).await.ok()
}

async fn seed_company_user(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    company_id: Uuid,
    company_name: &str,
    inviter_id: Uuid,
) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query(
        r#"
        INSERT INTO companies (id, name, company_email, country, timezone, status)
        VALUES ($1, $2, $3, 'TR', 'UTC', 'active')
        "#,
    )
    .bind(company_id)
    .bind(company_name)
    .bind(format!("{company_name}-{}@example.test", Uuid::new_v4()))
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO users (id, fullname, email, password_hash, account_type, role, status)
        VALUES ($1, 'Inviter', $2, 'hash', 'company', 'company_admin', 'active')
        "#,
    )
    .bind(inviter_id)
    .bind(format!("inviter-{}@example.test", Uuid::new_v4()))
    .execute(&mut **tx)
    .await?;

    sqlx::query("INSERT INTO company_users (company_id, user_id) VALUES ($1, $2)")
        .bind(company_id)
        .bind(inviter_id)
        .execute(&mut **tx)
        .await?;

    Ok(())
}

async fn insert_invitation(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    id: Uuid,
    company_id: Uuid,
    inviter_id: Uuid,
    email: &str,
    status: &str,
    expires_at: chrono::DateTime<Utc>,
    created_at: chrono::DateTime<Utc>,
) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query(
        r#"
        INSERT INTO invitations (
            id, invitation_token_hash, company_id, project_id,
            invited_email, invited_role, invited_by_user_id, status, expires_at, created_at
        )
        VALUES ($1, $2, $3, NULL, $4, 'contributor', $5, $6, $7, $8)
        "#,
    )
    .bind(id)
    .bind("c".repeat(64))
    .bind(company_id)
    .bind(email)
    .bind(inviter_id)
    .bind(status)
    .bind(expires_at)
    .bind(created_at)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Mirrors admin list default filter: pending + expired only when status param absent.
async fn admin_list_ids(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    company_id: Option<Uuid>,
    status: Option<&str>,
) -> Result<Vec<Uuid>, Box<dyn std::error::Error>> {
    let rows: Vec<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT i.id
        FROM invitations i
        JOIN companies c ON c.id = i.company_id
        JOIN users u ON u.id = i.invited_by_user_id
        WHERE ($1::uuid IS NULL OR i.company_id = $1)
          AND (
              ($2::text IS NOT NULL AND i.status = $2)
              OR ($2::text IS NULL AND i.status IN ('pending', 'expired'))
          )
        ORDER BY i.created_at DESC
        "#,
    )
    .bind(company_id)
    .bind(status)
    .fetch_all(&mut **tx)
    .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

/// Mirrors admin cancel: no company or inviter predicate.
async fn admin_cancel(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    invitation_id: Uuid,
) -> Result<Option<Uuid>, Box<dyn std::error::Error>> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        r#"
        UPDATE invitations i
        SET status = 'cancelled'
        FROM companies c, users u
        WHERE i.id = $1
          AND i.status = 'pending'
          AND c.id = i.company_id
          AND u.id = i.invited_by_user_id
        RETURNING i.id
        "#,
    )
    .bind(invitation_id)
    .fetch_optional(&mut **tx)
    .await?;
    Ok(row.map(|r| r.0))
}

#[tokio::test]
async fn admin_invitations_default_list_and_filters() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };

    sqlx::migrate!("./migrations").run(&pool).await?;

    let mut tx = pool.begin().await?;

    let company_a = Uuid::new_v4();
    let company_b = Uuid::new_v4();
    let inviter_a = Uuid::new_v4();
    let inviter_b = Uuid::new_v4();

    seed_company_user(&mut tx, company_a, "Co Alpha", inviter_a).await?;
    seed_company_user(&mut tx, company_b, "Co Beta", inviter_b).await?;

    let now = Utc::now();
    let pending_id = Uuid::new_v4();
    let expired_id = Uuid::new_v4();
    let accepted_id = Uuid::new_v4();
    let cancelled_id = Uuid::new_v4();
    let other_pending_id = Uuid::new_v4();

    insert_invitation(
        &mut tx,
        pending_id,
        company_a,
        inviter_a,
        "pending@example.test",
        "pending",
        now + ChronoDuration::hours(24),
        now,
    )
    .await?;

    insert_invitation(
        &mut tx,
        expired_id,
        company_a,
        inviter_a,
        "expired@example.test",
        "expired",
        now - ChronoDuration::hours(1),
        now - ChronoDuration::hours(48),
    )
    .await?;

    insert_invitation(
        &mut tx,
        accepted_id,
        company_a,
        inviter_a,
        "accepted@example.test",
        "accepted",
        now + ChronoDuration::hours(1),
        now - ChronoDuration::hours(24),
    )
    .await?;

    insert_invitation(
        &mut tx,
        cancelled_id,
        company_a,
        inviter_a,
        "cancelled@example.test",
        "cancelled",
        now + ChronoDuration::hours(1),
        now - ChronoDuration::hours(12),
    )
    .await?;

    insert_invitation(
        &mut tx,
        other_pending_id,
        company_b,
        inviter_b,
        "other@example.test",
        "pending",
        now + ChronoDuration::hours(24),
        now,
    )
    .await?;

    let default_ids = admin_list_ids(&mut tx, None, None).await?;
    assert!(default_ids.contains(&pending_id));
    assert!(default_ids.contains(&expired_id));
    assert!(default_ids.contains(&other_pending_id));
    assert!(!default_ids.contains(&accepted_id));
    assert!(!default_ids.contains(&cancelled_id));

    let accepted_only = admin_list_ids(&mut tx, None, Some("accepted")).await?;
    assert_eq!(accepted_only, vec![accepted_id]);

    let company_a_only = admin_list_ids(&mut tx, Some(company_a), None).await?;
    assert!(company_a_only.contains(&pending_id));
    assert!(company_a_only.contains(&expired_id));
    assert!(!company_a_only.contains(&other_pending_id));

    tx.rollback().await?;
    Ok(())
}

#[tokio::test]
async fn admin_cancel_pending_only_without_inviter_scope() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };

    sqlx::migrate!("./migrations").run(&pool).await?;

    let mut tx = pool.begin().await?;

    let company_id = Uuid::new_v4();
    let inviter_id = Uuid::new_v4();
    let other_user_id = Uuid::new_v4();

    seed_company_user(&mut tx, company_id, "Co Cancel", inviter_id).await?;

    sqlx::query(
        r#"
        INSERT INTO users (id, fullname, email, password_hash, account_type, role, status)
        VALUES ($1, 'Other', $2, 'hash', 'company', 'contributor', 'active')
        "#,
    )
    .bind(other_user_id)
    .bind(format!("other-{}@example.test", Uuid::new_v4()))
    .execute(&mut *tx)
    .await?;

    sqlx::query("INSERT INTO company_users (company_id, user_id) VALUES ($1, $2)")
        .bind(company_id)
        .bind(other_user_id)
        .execute(&mut *tx)
        .await?;

    let now = Utc::now();
    let pending_id = Uuid::new_v4();
    let expired_id = Uuid::new_v4();

    insert_invitation(
        &mut tx,
        pending_id,
        company_id,
        inviter_id,
        "cancel-me@example.test",
        "pending",
        now + ChronoDuration::hours(24),
        now,
    )
    .await?;

    insert_invitation(
        &mut tx,
        expired_id,
        company_id,
        inviter_id,
        "expired@example.test",
        "expired",
        now - ChronoDuration::hours(1),
        now,
    )
    .await?;

    let cancelled = admin_cancel(&mut tx, pending_id).await?;
    assert_eq!(cancelled, Some(pending_id));

    let status: (String,) =
        sqlx::query_as("SELECT status FROM invitations WHERE id = $1")
            .bind(pending_id)
            .fetch_one(&mut *tx)
            .await?;
    assert_eq!(status.0, "cancelled");

    let expired_cancel = admin_cancel(&mut tx, expired_id).await?;
    assert!(expired_cancel.is_none());

    tx.rollback().await?;
    Ok(())
}

#[tokio::test]
async fn admin_list_row_has_no_token_hash_column() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };

    sqlx::migrate!("./migrations").run(&pool).await?;

    let mut tx = pool.begin().await?;

    let company_id = Uuid::new_v4();
    let inviter_id = Uuid::new_v4();
    let invitation_id = Uuid::new_v4();

    seed_company_user(&mut tx, company_id, "Co Token", inviter_id).await?;

    let token_hash = "deadbeef".repeat(8);
    insert_invitation(
        &mut tx,
        invitation_id,
        company_id,
        inviter_id,
        "token-check@example.test",
        "pending",
        Utc::now() + ChronoDuration::hours(24),
        Utc::now(),
    )
    .await?;

    sqlx::query("UPDATE invitations SET invitation_token_hash = $1 WHERE id = $2")
        .bind(&token_hash)
        .bind(invitation_id)
        .execute(&mut *tx)
        .await?;

    let row: serde_json::Value = sqlx::query_scalar(
        r#"
        SELECT row_to_json(t)
        FROM (
            SELECT
                i.id,
                i.company_id,
                c.name AS company_name,
                i.invited_by_user_id,
                u.fullname AS inviter_name,
                u.email AS inviter_email,
                i.invited_email,
                i.invited_role,
                i.status,
                i.expires_at,
                i.created_at
            FROM invitations i
            JOIN companies c ON c.id = i.company_id
            JOIN users u ON u.id = i.invited_by_user_id
            WHERE i.id = $1
        ) t
        "#,
    )
    .bind(invitation_id)
    .fetch_one(&mut *tx)
    .await?;

    let obj = row.as_object().expect("json object");
    assert!(!obj.contains_key("invitation_token_hash"));
    assert!(!obj.contains_key("invitationTokenHash"));
    assert!(!obj.contains_key("token"));

    let serialized = serde_json::to_string(&row)?;
    assert!(!serialized.contains(&token_hash));

    tx.rollback().await?;
    Ok(())
}

#[tokio::test]
async fn system_admin_session_distinct_from_company_user() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };

    sqlx::migrate!("./migrations").run(&pool).await?;

    let mut tx = pool.begin().await?;

    let user_id = Uuid::new_v4();
    let company_id = Uuid::new_v4();

    seed_company_user(&mut tx, company_id, "Co Session", user_id).await?;

    let user_session_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO sessions (id, user_id, is_system_admin, system_admin_email, expires_at)
        VALUES ($1, $2, false, NULL, now() + interval '1 day')
        "#,
    )
    .bind(user_session_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    let admin_session_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO sessions (id, user_id, is_system_admin, system_admin_email, expires_at)
        VALUES ($1, NULL, true, 'admin@example.test', now() + interval '1 day')
        "#,
    )
    .bind(admin_session_id)
    .execute(&mut *tx)
    .await?;

    let user_row: (bool, Option<String>) = sqlx::query_as(
        "SELECT is_system_admin, system_admin_email FROM sessions WHERE id = $1",
    )
    .bind(user_session_id)
    .fetch_one(&mut *tx)
    .await?;
    assert!(!user_row.0);
    assert!(user_row.1.is_none());

    let admin_row: (bool, Option<String>) = sqlx::query_as(
        "SELECT is_system_admin, system_admin_email FROM sessions WHERE id = $1",
    )
    .bind(admin_session_id)
    .fetch_one(&mut *tx)
    .await?;
    assert!(admin_row.0);
    assert_eq!(admin_row.1.as_deref(), Some("admin@example.test"));

    tx.rollback().await?;
    Ok(())
}
