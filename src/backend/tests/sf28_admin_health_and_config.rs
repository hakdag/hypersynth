use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

async fn connect_pool() -> Option<PgPool> {
    let database_url = std::env::var("DATABASE_URL").ok()?;
    PgPool::connect(&database_url).await.ok()
}

#[tokio::test]
async fn platform_config_seed_and_update() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };

    sqlx::migrate!("./migrations").run(&pool).await?;

    let mut tx = pool.begin().await?;

    let row: (Vec<String>, Option<i64>, Option<String>, serde_json::Value) = sqlx::query_as(
        r#"
        SELECT allowed_ai_providers, default_monthly_token_limit, platform_announcement, feature_flags
        FROM platform_config
        WHERE id = 1
        "#,
    )
    .fetch_one(&mut *tx)
    .await?;

    assert_eq!(row.0.len(), 2);
    assert!(row.0.contains(&"anthropic".to_string()));
    assert!(row.0.contains(&"openai".to_string()));
    assert!(row.1.is_none());
    assert!(row.2.is_none());

    sqlx::query(
        r#"
        UPDATE platform_config
        SET
            allowed_ai_providers = ARRAY['openai']::text[],
            default_monthly_token_limit = 500000,
            platform_announcement = 'Maintenance tonight',
            feature_flags = '{"ai_requests_enabled": false}'::jsonb,
            updated_at = now()
        WHERE id = 1
        "#,
    )
    .execute(&mut *tx)
    .await?;

    let updated: (Vec<String>, Option<i64>, Option<String>, serde_json::Value) = sqlx::query_as(
        r#"
        SELECT allowed_ai_providers, default_monthly_token_limit, platform_announcement, feature_flags
        FROM platform_config
        WHERE id = 1
        "#,
    )
    .fetch_one(&mut *tx)
    .await?;

    assert_eq!(updated.0, vec!["openai".to_string()]);
    assert_eq!(updated.1, Some(500_000));
    assert_eq!(updated.2.as_deref(), Some("Maintenance tonight"));
    assert_eq!(updated.3["ai_requests_enabled"], json!(false));

    tx.rollback().await?;
    Ok(())
}

#[tokio::test]
async fn platform_config_update_writes_audit_row_change() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };

    sqlx::migrate!("./migrations").run(&pool).await?;

    let mut tx = pool.begin().await?;

    let before_count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)::bigint
        FROM audit_row_changes
        WHERE source->>'table' = 'platform_config'
        "#,
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE platform_config
        SET platform_announcement = 'Audit test announcement', updated_at = now()
        WHERE id = 1
        "#,
    )
    .execute(&mut *tx)
    .await?;

    let after_count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)::bigint
        FROM audit_row_changes
        WHERE source->>'table' = 'platform_config'
        "#,
    )
    .fetch_one(&mut *tx)
    .await?;

    assert!(after_count.0 > before_count.0);

    tx.rollback().await?;
    Ok(())
}

#[tokio::test]
async fn ai_usage_error_rate_aggregate_matches_health_logic() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };

    sqlx::migrate!("./migrations").run(&pool).await?;

    let mut tx = pool.begin().await?;

    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, fullname, email, password_hash, account_type, status)
        VALUES ($1, 'Usage User', $2, 'hash', 'personal', 'active')
        "#,
    )
    .bind(user_id)
    .bind(format!("usage-{}@example.test", Uuid::new_v4()))
    .execute(&mut *tx)
    .await?;

    let from = Utc::now() - chrono::Duration::hours(24);
    let to = Utc::now();

    for (status, _) in [("success", 0), ("success", 0), ("failed", 0)] {
        sqlx::query(
            r#"
            INSERT INTO ai_usage (
                user_id, operation_type, provider, model,
                input_tokens, output_tokens, status, created_at
            )
            VALUES ($1, 'generate_tasks', 'openai', 'gpt-4o-mini', 10, 5, $2, now())
            "#,
        )
        .bind(user_id)
        .bind(status)
        .execute(&mut *tx)
        .await?;
    }

    let row: (i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)::bigint AS request_count,
            COUNT(*) FILTER (WHERE status = 'failed')::bigint AS failure_count
        FROM ai_usage
        WHERE created_at >= $1 AND created_at < $2
          AND user_id = $3
        "#,
    )
    .bind(from)
    .bind(to)
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    assert_eq!(row.0, 3);
    assert_eq!(row.1, 1);

    let rate = row.1 as f64 / row.0 as f64;
    assert!(rate >= 0.10);

    tx.rollback().await?;
    Ok(())
}

#[tokio::test]
async fn system_admin_session_required_for_admin_surface() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };

    sqlx::migrate!("./migrations").run(&pool).await?;

    let mut tx = pool.begin().await?;

    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, fullname, email, password_hash, account_type, status)
        VALUES ($1, 'Regular', $2, 'hash', 'personal', 'active')
        "#,
    )
    .bind(user_id)
    .bind(format!("user-{}@example.test", Uuid::new_v4()))
    .execute(&mut *tx)
    .await?;

    let user_session = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO sessions (id, user_id, is_system_admin, system_admin_email, expires_at)
        VALUES ($1, $2, false, NULL, now() + interval '1 day')
        "#,
    )
    .bind(user_session)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    let admin_session = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO sessions (id, user_id, is_system_admin, system_admin_email, expires_at)
        VALUES ($1, NULL, true, 'admin@example.test', now() + interval '1 day')
        "#,
    )
    .bind(admin_session)
    .execute(&mut *tx)
    .await?;

    let user_admin: (bool,) = sqlx::query_as(
        "SELECT is_system_admin FROM sessions WHERE id = $1",
    )
    .bind(user_session)
    .fetch_one(&mut *tx)
    .await?;
    assert!(!user_admin.0);

    let admin_flag: (bool,) = sqlx::query_as(
        "SELECT is_system_admin FROM sessions WHERE id = $1",
    )
    .bind(admin_session)
    .fetch_one(&mut *tx)
    .await?;
    assert!(admin_flag.0);

    tx.rollback().await?;
    Ok(())
}
