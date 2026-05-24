use chrono::{TimeZone, Utc};
use serde_json::Value;
use sqlx::{FromRow, PgConnection};
use uuid::Uuid;

use crate::ai_usage_query_helpers::{pagination_limit, pagination_offset};
use crate::types::{AdminAuditLogEntry, AdminAuditLogsListQuery, AdminAuditLogsListResponse};

const SENSITIVE_KEYS: &[&str] = &[
    "password_hash",
    "encrypted_api_key",
    "token_hash",
    "invitation_token_hash",
];

/// Normalizes SF-24's dual audit stores into FRD-shaped entries for the
/// System Admin viewer (SF-26).
pub struct AuditLogQueryService;

#[derive(Debug, FromRow)]
struct AuditLogRawRow {
    entry_id: String,
    ts_ms: i64,
    system_admin_email: Option<String>,
    user_id: Option<Uuid>,
    company_id: Option<Uuid>,
    action_type: String,
    entity_type: String,
    entity_id: Option<String>,
    metadata: Value,
    ip_address: Option<String>,
    user_agent: Option<String>,
}

const UNIFIED_AUDIT_CTE: &str = r#"
WITH unified AS (
    SELECT
        'rc:' || rc.id::text AS entry_id,
        rc.ts_ms,
        CASE
            WHEN COALESCE((rc.actor->>'system_admin')::boolean, false) THEN rc.actor->>'email'
            ELSE NULL
        END AS system_admin_email,
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
        END AS action_type,
        rc.source->>'table' AS entity_type,
        COALESCE(rc."after"->>'id', rc."before"->>'id') AS entity_id,
        jsonb_build_object(
            'op', rc.op,
            'source', rc.source,
            'before', rc."before",
            'after', rc."after"
        ) AS metadata,
        host(rc.ip_address) AS ip_address,
        rc.user_agent
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
        CASE
            WHEN COALESCE((ev.actor->>'system_admin')::boolean, false) THEN ev.actor->>'email'
            ELSE NULL
        END AS system_admin_email,
        NULLIF(ev.actor->>'user_id', '')::uuid AS user_id,
        COALESCE(
            NULLIF(ev.actor->>'company_id', '')::uuid,
            NULLIF(ev.payload->>'company_id', '')::uuid
        ) AS company_id,
        ev.event_type AS action_type,
        COALESCE(NULLIF(ev.payload->>'entity_type', ''), 'system') AS entity_type,
        NULLIF(ev.payload->>'entity_id', '') AS entity_id,
        ev.payload AS metadata,
        host(ev.ip_address) AS ip_address,
        ev.user_agent
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
"#;

impl AuditLogQueryService {
    pub async fn list(
        conn: &mut PgConnection,
        query: &AdminAuditLogsListQuery,
    ) -> Result<AdminAuditLogsListResponse, sqlx::Error> {
        let limit = pagination_limit(query.limit);
        let offset = pagination_offset(query.offset);

        let from_ms = query.from.map(|t| t.timestamp_millis());
        let to_ms = query.to.map(|t| t.timestamp_millis());

        let action_type = query
            .action_type
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);

        let total: i64 = sqlx::query_scalar(&format!(
            "{UNIFIED_AUDIT_CTE} SELECT COUNT(*)::bigint FROM unified"
        ))
        .bind(from_ms)
        .bind(to_ms)
        .bind(query.user_id)
        .bind(query.company_id)
        .bind(action_type.as_deref())
        .fetch_one(&mut *conn)
        .await?;

        let rows = sqlx::query_as::<_, AuditLogRawRow>(&format!(
            "{UNIFIED_AUDIT_CTE}
             SELECT
                 entry_id,
                 ts_ms,
                 system_admin_email,
                 user_id,
                 company_id,
                 action_type,
                 entity_type,
                 entity_id,
                 metadata,
                 ip_address,
                 user_agent
             FROM unified
             ORDER BY ts_ms DESC
             LIMIT $6 OFFSET $7"
        ))
        .bind(from_ms)
        .bind(to_ms)
        .bind(query.user_id)
        .bind(query.company_id)
        .bind(action_type.as_deref())
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut *conn)
        .await?;

        let items = rows.into_iter().map(raw_row_to_entry).collect();

        Ok(AdminAuditLogsListResponse {
            items,
            total,
            limit,
            offset,
        })
    }
}

fn raw_row_to_entry(row: AuditLogRawRow) -> AdminAuditLogEntry {
    let created_at = Utc
        .timestamp_millis_opt(row.ts_ms)
        .single()
        .unwrap_or_else(Utc::now);

    AdminAuditLogEntry {
        id: row.entry_id,
        created_at,
        company_id: row.company_id,
        user_id: row.user_id,
        system_admin_email: row.system_admin_email,
        action_type: row.action_type,
        entity_type: row.entity_type,
        entity_id: row.entity_id,
        metadata: redact_sensitive_metadata(row.metadata),
        ip_address: row.ip_address,
        user_agent: row.user_agent,
    }
}

fn redact_sensitive_metadata(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (key, val) in map {
                if SENSITIVE_KEYS.contains(&key.as_str()) {
                    continue;
                }
                out.insert(key, redact_sensitive_metadata(val));
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(redact_sensitive_metadata).collect()),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn redact_sensitive_metadata_strips_nested_keys() {
        let input = json!({
            "before": { "password_hash": "secret", "email": "a@b.c" },
            "after": { "encrypted_api_key": "key", "name": "x" },
            "token_hash": "t"
        });
        let out = redact_sensitive_metadata(input);
        assert!(out.get("token_hash").is_none());
        assert_eq!(out["before"]["email"], "a@b.c");
        assert!(out["before"].get("password_hash").is_none());
        assert_eq!(out["after"]["name"], "x");
        assert!(out["after"].get("encrypted_api_key").is_none());
    }
}
