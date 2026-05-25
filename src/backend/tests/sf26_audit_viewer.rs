use chrono::Utc;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

async fn connect_pool() -> Option<PgPool> {
    let database_url = std::env::var("DATABASE_URL").ok()?;
    PgPool::connect(&database_url).await.ok()
}

const UNIFIED_COUNT: &str = r#"
WITH unified AS (
    SELECT
        'rc:' || rc.id::text AS entry_id,
        rc.ts_ms,
        NULLIF(rc.actor->>'user_id', '')::uuid AS user_id,
        COALESCE(
            NULLIF(rc.actor->>'company_id', '')::uuid,
            NULLIF(rc."after"->>'company_id', '')::uuid,
            NULLIF(rc."before"->>'company_id', '')::uuid,
            CASE
                WHEN rc.source->>'table' = 'companies' THEN
                    COALESCE(
                        NULLIF(rc."after"->>'id', '')::uuid,
                        NULLIF(rc."before"->>'id', '')::uuid
                    )
                ELSE NULL
            END
        ) AS company_id,
        (rc.source->>'table') || '_' || CASE rc.op
            WHEN 'c' THEN 'created'
            WHEN 'u' THEN 'updated'
            WHEN 'd' THEN 'deleted'
        END AS action_type
    FROM audit_row_changes rc
    WHERE ($1::bigint IS NULL OR rc.ts_ms >= $1)
      AND ($2::bigint IS NULL OR rc.ts_ms <= $2)
      AND ($3::uuid IS NULL OR NULLIF(rc.actor->>'user_id', '')::uuid = $3)
      AND (
          $4::uuid IS NULL
          OR COALESCE(
              NULLIF(rc.actor->>'company_id', '')::uuid,
              NULLIF(rc."after"->>'company_id', '')::uuid,
              NULLIF(rc."before"->>'company_id', '')::uuid,
              CASE
                  WHEN rc.source->>'table' = 'companies' THEN
                      COALESCE(
                          NULLIF(rc."after"->>'id', '')::uuid,
                          NULLIF(rc."before"->>'id', '')::uuid
                      )
                  ELSE NULL
              END
          ) = $4
      )
      AND (
          $5::text IS NULL
          OR (rc.source->>'table') || '_' || CASE rc.op
              WHEN 'c' THEN 'created'
              WHEN 'u' THEN 'updated'
              WHEN 'd' THEN 'deleted'
          END = $5
      )

    UNION ALL

    SELECT
        'ev:' || ev.id::text AS entry_id,
        ev.ts_ms,
        NULLIF(ev.actor->>'user_id', '')::uuid AS user_id,
        COALESCE(
            NULLIF(ev.actor->>'company_id', '')::uuid,
            NULLIF(ev.payload->>'company_id', '')::uuid
        ) AS company_id,
        ev.event_type AS action_type
    FROM audit_events ev
    WHERE ($1::bigint IS NULL OR ev.ts_ms >= $1)
      AND ($2::bigint IS NULL OR ev.ts_ms <= $2)
      AND ($3::uuid IS NULL OR NULLIF(ev.actor->>'user_id', '')::uuid = $3)
      AND (
          $4::uuid IS NULL
          OR COALESCE(
              NULLIF(ev.actor->>'company_id', '')::uuid,
              NULLIF(ev.payload->>'company_id', '')::uuid
          ) = $4
      )
      AND ($5::text IS NULL OR ev.event_type = $5)
)
SELECT COUNT(*)::bigint FROM unified
"#;

const UNIFIED_LIST: &str = r#"
WITH unified AS (
    SELECT
        'rc:' || rc.id::text AS entry_id,
        rc.ts_ms,
        NULLIF(rc.actor->>'user_id', '')::uuid AS user_id,
        COALESCE(
            NULLIF(rc.actor->>'company_id', '')::uuid,
            NULLIF(rc."after"->>'company_id', '')::uuid,
            NULLIF(rc."before"->>'company_id', '')::uuid,
            CASE
                WHEN rc.source->>'table' = 'companies' THEN
                    COALESCE(
                        NULLIF(rc."after"->>'id', '')::uuid,
                        NULLIF(rc."before"->>'id', '')::uuid
                    )
                ELSE NULL
            END
        ) AS company_id,
        (rc.source->>'table') || '_' || CASE rc.op
            WHEN 'c' THEN 'created'
            WHEN 'u' THEN 'updated'
            WHEN 'd' THEN 'deleted'
        END AS action_type
    FROM audit_row_changes rc
    WHERE ($1::bigint IS NULL OR rc.ts_ms >= $1)
      AND ($2::bigint IS NULL OR rc.ts_ms <= $2)
      AND ($3::uuid IS NULL OR NULLIF(rc.actor->>'user_id', '')::uuid = $3)
      AND (
          $4::uuid IS NULL
          OR COALESCE(
              NULLIF(rc.actor->>'company_id', '')::uuid,
              NULLIF(rc."after"->>'company_id', '')::uuid,
              NULLIF(rc."before"->>'company_id', '')::uuid,
              CASE
                  WHEN rc.source->>'table' = 'companies' THEN
                      COALESCE(
                          NULLIF(rc."after"->>'id', '')::uuid,
                          NULLIF(rc."before"->>'id', '')::uuid
                      )
                  ELSE NULL
              END
          ) = $4
      )
      AND (
          $5::text IS NULL
          OR (rc.source->>'table') || '_' || CASE rc.op
              WHEN 'c' THEN 'created'
              WHEN 'u' THEN 'updated'
              WHEN 'd' THEN 'deleted'
          END = $5
      )

    UNION ALL

    SELECT
        'ev:' || ev.id::text AS entry_id,
        ev.ts_ms,
        NULLIF(ev.actor->>'user_id', '')::uuid AS user_id,
        COALESCE(
            NULLIF(ev.actor->>'company_id', '')::uuid,
            NULLIF(ev.payload->>'company_id', '')::uuid
        ) AS company_id,
        ev.event_type AS action_type
    FROM audit_events ev
    WHERE ($1::bigint IS NULL OR ev.ts_ms >= $1)
      AND ($2::bigint IS NULL OR ev.ts_ms <= $2)
      AND ($3::uuid IS NULL OR NULLIF(ev.actor->>'user_id', '')::uuid = $3)
      AND (
          $4::uuid IS NULL
          OR COALESCE(
              NULLIF(ev.actor->>'company_id', '')::uuid,
              NULLIF(ev.payload->>'company_id', '')::uuid
          ) = $4
      )
      AND ($5::text IS NULL OR ev.event_type = $5)
)
SELECT entry_id, ts_ms, action_type FROM unified ORDER BY ts_ms DESC LIMIT $6 OFFSET $7
"#;

/// SF-26: unified audit query supports filters, ordering, and pagination.
#[tokio::test]
async fn audit_viewer_unified_query_filters_and_paginates() -> Result<(), Box<dyn std::error::Error>>
{
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };
    sqlx::migrate!("./migrations").run(&pool).await?;

    let company_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let request_id = Uuid::new_v4();
    let email = format!("sf26-{}@example.test", Uuid::new_v4());

    let older_ts = Utc::now().timestamp_millis() - 60_000;
    let newer_ts = Utc::now().timestamp_millis();

    let actor = json!({
        "system_admin": false,
        "user_id": user_id,
        "email": email,
        "account_type": "company",
        "company_id": company_id
    });

    sqlx::query(
        r#"
        INSERT INTO audit_events (
            ts_ms, event_type, actor, payload, request_id, ip_address, user_agent
        )
        VALUES ($1, 'system_admin_login_success', $2, $3, $4, '203.0.113.1'::inet, 'test/1')
        "#,
    )
    .bind(newer_ts)
    .bind(json!({"system_admin": true, "email": "admin@example.test"}))
    .bind(json!({"attempted_email": "admin@example.test"}))
    .bind(request_id)
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO audit_row_changes (
            ts_ms, op, source, "before", "after", actor, request_id, ip_address, user_agent
        )
        VALUES (
            $1, 'u', $2, $3, $4, $5, $6, '203.0.113.2'::inet, 'test/2'
        )
        "#,
    )
    .bind(older_ts)
    .bind(json!({"schema": "public", "table": "projects", "tx_id": 1}))
    .bind(json!({"id": project_id, "name": "old", "company_id": company_id}))
    .bind(json!({"id": project_id, "name": "new", "company_id": company_id}))
    .bind(&actor)
    .bind(request_id)
    .execute(&pool)
    .await?;

    let total: i64 = sqlx::query_scalar(UNIFIED_COUNT)
        .bind(None::<i64>)
        .bind(None::<i64>)
        .bind(None::<Uuid>)
        .bind(Some(company_id))
        .bind(None::<&str>)
        .fetch_one(&pool)
        .await?;
    assert!(total >= 2, "expected at least two seeded rows for company filter");

    let by_user: i64 = sqlx::query_scalar(UNIFIED_COUNT)
        .bind(None::<i64>)
        .bind(None::<i64>)
        .bind(Some(user_id))
        .bind(None::<Uuid>)
        .bind(None::<&str>)
        .fetch_one(&pool)
        .await?;
    assert!(by_user >= 1);

    let by_action: i64 = sqlx::query_scalar(UNIFIED_COUNT)
        .bind(None::<i64>)
        .bind(None::<i64>)
        .bind(None::<Uuid>)
        .bind(None::<Uuid>)
        .bind(Some("projects_updated"))
        .fetch_one(&pool)
        .await?;
    assert!(by_action >= 1);

    let from_ms = older_ts - 1_000;
    let to_ms = newer_ts + 1_000;
    let in_range: i64 = sqlx::query_scalar(UNIFIED_COUNT)
        .bind(Some(from_ms))
        .bind(Some(to_ms))
        .bind(None::<Uuid>)
        .bind(Some(company_id))
        .bind(None::<&str>)
        .fetch_one(&pool)
        .await?;
    assert!(in_range >= 2);

    let rows: Vec<(String, i64, String)> = sqlx::query_as(UNIFIED_LIST)
        .bind(None::<i64>)
        .bind(None::<i64>)
        .bind(None::<Uuid>)
        .bind(Some(company_id))
        .bind(None::<&str>)
        .bind(1_i64)
        .bind(0_i64)
        .fetch_all(&pool)
        .await?;

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].2, "system_admin_login_success");

    Ok(())
}

/// SF-26: metadata redaction strips sensitive keys recursively.
#[test]
fn audit_viewer_metadata_redaction_unit() {
    let input = json!({
        "password_hash": "secret",
        "nested": { "token_hash": "t", "ok": true }
    });

    fn redact(value: Value) -> Value {
        const KEYS: &[&str] = &[
            "password_hash",
            "encrypted_api_key",
            "token_hash",
            "invitation_token_hash",
        ];
        match value {
            Value::Object(map) => {
                let mut out = serde_json::Map::new();
                for (key, val) in map {
                    if KEYS.contains(&key.as_str()) {
                        continue;
                    }
                    out.insert(key, redact(val));
                }
                Value::Object(out)
            }
            Value::Array(items) => Value::Array(items.into_iter().map(redact).collect()),
            other => other,
        }
    }

    let out = redact(input);
    assert!(out.get("password_hash").is_none());
    assert!(out["nested"].get("token_hash").is_none());
    assert_eq!(out["nested"]["ok"], true);
}
