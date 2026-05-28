use std::collections::{HashMap, HashSet};

use axum::http::StatusCode;
use axum::Json;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::types::{ApiErrorBody, TenantScope};

pub const MAX_LABEL_NAME_LEN: usize = 100;

#[derive(Debug, Clone)]
pub struct TaskLabelSummaryRow {
    pub id: Uuid,
    pub name: String,
    pub color: String,
}

pub fn normalize_hex_color(raw: &str) -> Result<String, (StatusCode, Json<ApiErrorBody>)> {
    let value = raw.trim();
    if value.len() != 7 || !value.starts_with('#') {
        return Err(bad_request("Color must be a hex value in the format #RRGGBB."));
    }

    if !value[1..].chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(bad_request("Color must be a hex value in the format #RRGGBB."));
    }

    Ok(format!("#{}", value[1..].to_ascii_uppercase()))
}

pub fn normalize_label_name(raw: &str) -> Result<String, (StatusCode, Json<ApiErrorBody>)> {
    let value = raw.trim();
    if value.is_empty() {
        return Err(bad_request("Label name is required."));
    }

    if value.len() > MAX_LABEL_NAME_LEN {
        return Err(bad_request("Label name is too long."));
    }

    Ok(value.to_string())
}

pub async fn validate_label_ids_for_task(
    conn: &mut PgConnection,
    scope: TenantScope,
    project_id: Uuid,
    label_ids: &[Uuid],
) -> Result<Vec<Uuid>, (StatusCode, Json<ApiErrorBody>)> {
    let deduped = dedupe_label_ids(label_ids);
    if deduped.is_empty() {
        return Ok(deduped);
    }

    let expected_count = i64::try_from(deduped.len()).map_err(|_| internal_error())?;
    let linked_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)::bigint
        FROM labels l
        INNER JOIN projects p ON p.id = $3
        WHERE l.id = ANY($1)
          AND (
            ($2::uuid IS NOT NULL AND l.company_id = $2 AND p.company_id = $2)
            OR
            ($4::uuid IS NOT NULL AND l.user_id = $4 AND p.owner_user_id = $4 AND p.company_id IS NULL)
          )
        "#,
    )
    .bind(&deduped)
    .bind(scope.company_id_or_null())
    .bind(project_id)
    .bind(scope.owner_user_id_or_null())
    .fetch_one(&mut *conn)
    .await
    .map_err(|_| internal_error())?;

    if linked_count != expected_count {
        return Err(bad_request("That label cannot be applied to this task."));
    }

    Ok(deduped)
}

pub async fn sync_task_labels(
    conn: &mut PgConnection,
    task_id: Uuid,
    label_ids: &[Uuid],
) -> Result<(), (StatusCode, Json<ApiErrorBody>)> {
    let deduped = dedupe_label_ids(label_ids);
    sqlx::query("DELETE FROM task_labels WHERE task_id = $1")
        .bind(task_id)
        .execute(&mut *conn)
        .await
        .map_err(|_| internal_error())?;

    for label_id in deduped {
        sqlx::query(
            r#"
            INSERT INTO task_labels (task_id, label_id)
            VALUES ($1, $2)
            ON CONFLICT (task_id, label_id) DO NOTHING
            "#,
        )
        .bind(task_id)
        .bind(label_id)
        .execute(&mut *conn)
        .await
        .map_err(|_| internal_error())?;
    }

    Ok(())
}

pub async fn fetch_labels_for_tasks(
    conn: &mut PgConnection,
    task_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<TaskLabelSummaryRow>>, (StatusCode, Json<ApiErrorBody>)> {
    let ids = dedupe_label_ids(task_ids);
    if ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query_as::<_, (Uuid, Uuid, String, String)>(
        r#"
        SELECT tl.task_id, l.id, l.name, l.color
        FROM task_labels tl
        INNER JOIN labels l ON l.id = tl.label_id
        WHERE tl.task_id = ANY($1)
        ORDER BY lower(l.name) ASC
        "#,
    )
    .bind(&ids)
    .fetch_all(&mut *conn)
    .await
    .map_err(|_| internal_error())?;

    let mut out: HashMap<Uuid, Vec<TaskLabelSummaryRow>> = HashMap::new();
    for (task_id, id, name, color) in rows {
        out.entry(task_id)
            .or_default()
            .push(TaskLabelSummaryRow { id, name, color });
    }
    Ok(out)
}

fn dedupe_label_ids(ids: &[Uuid]) -> Vec<Uuid> {
    let mut seen = HashSet::new();
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        if seen.insert(*id) {
            out.push(*id);
        }
    }
    out
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
    use super::{normalize_hex_color, normalize_label_name};

    #[test]
    fn normalize_hex_color_accepts_valid_value() {
        let color = normalize_hex_color("#1a2B3c").expect("valid color");
        assert_eq!(color, "#1A2B3C");
    }

    #[test]
    fn normalize_hex_color_rejects_short_hex() {
        assert!(normalize_hex_color("#abc").is_err());
    }

    #[test]
    fn normalize_hex_color_rejects_non_hex_characters() {
        assert!(normalize_hex_color("#12GG34").is_err());
    }

    #[test]
    fn normalize_label_name_trims_value() {
        let name = normalize_label_name("  Backend  ").expect("valid name");
        assert_eq!(name, "Backend");
    }

    #[test]
    fn normalize_label_name_rejects_empty() {
        assert!(normalize_label_name("   ").is_err());
    }
}
