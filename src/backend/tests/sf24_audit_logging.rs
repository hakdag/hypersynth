use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

async fn connect_pool() -> Option<PgPool> {
    let database_url = std::env::var("DATABASE_URL").ok()?;
    PgPool::connect(&database_url).await.ok()
}

/// End-to-end check for the row-change trigger: when a request transaction
/// updates a projects row with the per-request GUCs set, an
/// `audit_row_changes` row should appear with op='u', a Debezium-shaped
/// envelope (source/before/after) and the actor/request_id values that the
/// "middleware" set.
#[tokio::test]
async fn projects_update_emits_debezium_envelope_with_actor_context() -> Result<(), Box<dyn std::error::Error>>
{
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };
    sqlx::migrate!("./migrations").run(&pool).await?;

    let user_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let request_id = Uuid::new_v4();
    let email = format!("audit-trigger-{}@example.test", Uuid::new_v4());

    let actor_json = serde_json::json!({
        "system_admin": false,
        "user_id": user_id,
        "email": email,
        "account_type": "personal",
    })
    .to_string();

    let mut conn = pool.acquire().await?;
    sqlx::query("BEGIN").execute(&mut *conn).await?;

    sqlx::query(
        "SELECT set_config('app.actor', $1, true),\
                set_config('app.request_id', $2, true),\
                set_config('app.ip_address', $3, true),\
                set_config('app.user_agent', $4, true)",
    )
    .bind(&actor_json)
    .bind(request_id.to_string())
    .bind("203.0.113.7")
    .bind("integration-test/1.0")
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        "INSERT INTO users (id, fullname, email, password_hash, account_type, role) \
         VALUES ($1, 'Audit Tester', $2, 'not-used', 'personal', NULL)",
    )
    .bind(user_id)
    .bind(&email)
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        "INSERT INTO projects (id, owner_user_id, company_id, created_by_user_id, name, status) \
         VALUES ($1, $2, NULL, $2, 'initial', 'Pending')",
    )
    .bind(project_id)
    .bind(user_id)
    .execute(&mut *conn)
    .await?;

    sqlx::query("UPDATE projects SET name = 'renamed' WHERE id = $1")
        .bind(project_id)
        .execute(&mut *conn)
        .await?;

    // Read back the most recent UPDATE on this row, inside the same tx so it
    // is visible even though we will roll back at the end of the test.
    let (op, source, before, after, actor, captured_request_id, ip, ua): (
        String,
        Value,
        Option<Value>,
        Option<Value>,
        Option<Value>,
        Option<Uuid>,
        Option<String>,
        Option<String>,
    ) = sqlx::query_as(
        "SELECT op, source, \"before\", \"after\", actor, request_id, \
                host(ip_address)::text, user_agent \
         FROM audit_row_changes \
         WHERE source->>'table' = 'projects' AND op = 'u' \
           AND (\"after\"->>'id')::uuid = $1 \
         ORDER BY ts_ms DESC LIMIT 1",
    )
    .bind(project_id)
    .fetch_one(&mut *conn)
    .await?;

    sqlx::query("ROLLBACK").execute(&mut *conn).await?;

    assert_eq!(op, "u");
    assert_eq!(source.get("table").and_then(Value::as_str), Some("projects"));
    let before = before.expect("UPDATE must capture before image");
    let after = after.expect("UPDATE must capture after image");
    assert_eq!(before.get("name").and_then(Value::as_str), Some("initial"));
    assert_eq!(after.get("name").and_then(Value::as_str), Some("renamed"));
    let actor = actor.expect("actor must be stamped from app.actor GUC");
    assert_eq!(
        actor.get("user_id").and_then(Value::as_str),
        Some(user_id.to_string().as_str())
    );
    assert_eq!(captured_request_id, Some(request_id));
    assert_eq!(ip.as_deref(), Some("203.0.113.7"));
    assert_eq!(ua.as_deref(), Some("integration-test/1.0"));

    Ok(())
}

/// Sensitive columns listed in `audit.masked_columns` must be replaced with
/// the literal string `***` in both `before` and `after`. We use
/// `users.password_hash` which is seeded by the SF-24 migration.
#[tokio::test]
async fn users_password_hash_is_masked_in_audit_rows() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };
    sqlx::migrate!("./migrations").run(&pool).await?;

    let user_id = Uuid::new_v4();
    let email = format!("audit-mask-{}@example.test", Uuid::new_v4());

    let mut conn = pool.acquire().await?;
    sqlx::query("BEGIN").execute(&mut *conn).await?;

    sqlx::query(
        "SELECT set_config('app.actor', $1, true), set_config('app.request_id', $2, true)",
    )
    .bind("{\"system_admin\":false}")
    .bind(Uuid::new_v4().to_string())
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        "INSERT INTO users (id, fullname, email, password_hash, account_type, role) \
         VALUES ($1, 'Mask Tester', $2, 'super-secret-hash', 'personal', NULL)",
    )
    .bind(user_id)
    .bind(&email)
    .execute(&mut *conn)
    .await?;

    let after: Value = sqlx::query_scalar(
        "SELECT \"after\" FROM audit_row_changes \
         WHERE source->>'table' = 'users' AND op = 'c' \
           AND (\"after\"->>'id')::uuid = $1 \
         ORDER BY ts_ms DESC LIMIT 1",
    )
    .bind(user_id)
    .fetch_one(&mut *conn)
    .await?;

    sqlx::query("ROLLBACK").execute(&mut *conn).await?;

    assert_eq!(
        after.get("password_hash").and_then(Value::as_str),
        Some("***"),
        "password_hash must be masked in audit_row_changes"
    );

    Ok(())
}

/// Audit tables are append-only at the SQL layer. Any attempt to UPDATE or
/// DELETE rows from `audit_row_changes` / `audit_events` must raise.
#[tokio::test]
async fn audit_tables_reject_update_and_delete() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };
    sqlx::migrate!("./migrations").run(&pool).await?;

    let update_err = sqlx::query("UPDATE audit_events SET event_type = 'x'")
        .execute(&pool)
        .await
        .expect_err("UPDATE on audit_events must fail");
    let delete_err = sqlx::query("DELETE FROM audit_row_changes")
        .execute(&pool)
        .await
        .expect_err("DELETE on audit_row_changes must fail");

    let combined = format!("{update_err}|{delete_err}");
    assert!(
        combined.to_lowercase().contains("append"),
        "expected append-only enforcement, got: {combined}"
    );

    Ok(())
}
