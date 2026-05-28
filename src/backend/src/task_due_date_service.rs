use axum::http::StatusCode;
use axum::Json;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::types::{ApiErrorBody, TaskDetailResponse, TaskResponse, TenantScope};

pub const TERMINAL_TASK_STATUSES: &[&str] = &["Done"];

pub fn resolve_due_fields(
    due_date_raw: Option<&str>,
    due_time_raw: Option<&str>,
    clear_due_date: bool,
    existing_due_date: Option<NaiveDate>,
    existing_due_time: Option<NaiveTime>,
) -> Result<(Option<NaiveDate>, Option<NaiveTime>), (StatusCode, Json<ApiErrorBody>)> {
    if clear_due_date {
        return Ok((None, None));
    }

    let due_date = parse_due_date(due_date_raw)?;
    let due_time = parse_due_time(due_time_raw)?;

    let resolved_due_date = due_date.or(existing_due_date);
    let resolved_due_time = if due_date.is_some() {
        due_time
    } else if due_time.is_some() {
        return Err(bad_request(
            "Due time cannot be set without a due date.",
        ));
    } else {
        existing_due_time
    };

    if resolved_due_date.is_none() && resolved_due_time.is_some() {
        return Err(bad_request(
            "Due time cannot be set without a due date.",
        ));
    }

    Ok((resolved_due_date, resolved_due_time))
}

pub fn compute_is_overdue(
    due_date: Option<NaiveDate>,
    due_time: Option<NaiveTime>,
    status: &str,
    workspace_now: NaiveDateTime,
) -> bool {
    if due_date.is_none() || TERMINAL_TASK_STATUSES.contains(&status) {
        return false;
    }

    let due_date = due_date.expect("due_date checked above");
    if let Some(due_time) = due_time {
        let due_at = NaiveDateTime::new(due_date, due_time);
        workspace_now > due_at
    } else {
        workspace_now.date() > due_date
    }
}

pub fn enrich_task_row(row: &mut TaskResponse, workspace_now: NaiveDateTime) {
    row.is_overdue = compute_is_overdue(row.due_date, row.due_time, &row.status, workspace_now);
}

pub fn enrich_task_detail_row(row: &mut TaskDetailResponse, workspace_now: NaiveDateTime) {
    row.is_overdue = compute_is_overdue(row.due_date, row.due_time, &row.status, workspace_now);
}

pub async fn resolve_workspace_now(
    conn: &mut PgConnection,
    scope: TenantScope,
    project_id: Uuid,
) -> Result<NaiveDateTime, (StatusCode, Json<ApiErrorBody>)> {
    let timezone_name = resolve_workspace_timezone(conn, scope, project_id).await?;
    sqlx::query_scalar::<_, NaiveDateTime>("SELECT timezone($1, now())::timestamp")
        .bind(timezone_name)
        .fetch_one(&mut *conn)
        .await
        .map_err(|_| internal_error())
}

async fn resolve_workspace_timezone(
    conn: &mut PgConnection,
    scope: TenantScope,
    project_id: Uuid,
) -> Result<String, (StatusCode, Json<ApiErrorBody>)> {
    match scope {
        TenantScope::Company { company_id, .. } => {
            sqlx::query_scalar::<_, String>(
                r#"
                SELECT c.timezone
                FROM projects p
                INNER JOIN companies c ON c.id = p.company_id
                WHERE p.id = $1 AND p.company_id = $2
                "#,
            )
            .bind(project_id)
            .bind(company_id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|_| internal_error())?
            .ok_or_else(|| not_found("Project not found."))
        }
        TenantScope::Personal { user_id } => {
            let timezone = sqlx::query_scalar::<_, Option<String>>(
                r#"
                SELECT u.timezone
                FROM users u
                WHERE u.id = $1
                "#,
            )
            .bind(user_id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|_| internal_error())?
            .flatten()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "UTC".to_string());
            Ok(timezone)
        }
    }
}

fn parse_due_date(
    due_date_raw: Option<&str>,
) -> Result<Option<NaiveDate>, (StatusCode, Json<ApiErrorBody>)> {
    let Some(raw) = due_date_raw.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    NaiveDate::parse_from_str(raw, "%Y-%m-%d")
        .map(Some)
        .map_err(|_| bad_request("Due date must use YYYY-MM-DD format."))
}

fn parse_due_time(
    due_time_raw: Option<&str>,
) -> Result<Option<NaiveTime>, (StatusCode, Json<ApiErrorBody>)> {
    let Some(raw) = due_time_raw.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    NaiveTime::parse_from_str(raw, "%H:%M")
        .or_else(|_| NaiveTime::parse_from_str(raw, "%H:%M:%S"))
        .map(Some)
        .map_err(|_| bad_request("Due time must use HH:MM or HH:MM:SS format."))
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

fn not_found(message: impl Into<String>) -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::NOT_FOUND,
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
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

    use crate::task_due_date_service::compute_is_overdue;

    #[test]
    fn overdue_false_without_due_date() {
        let now = NaiveDate::from_ymd_opt(2026, 5, 28)
            .expect("valid date")
            .and_hms_opt(10, 0, 0)
            .expect("valid time");
        assert!(!compute_is_overdue(None, None, "Pending", now));
    }

    #[test]
    fn overdue_false_for_terminal_status() {
        let now = NaiveDate::from_ymd_opt(2026, 5, 28)
            .expect("valid date")
            .and_hms_opt(10, 0, 0)
            .expect("valid time");
        let due_date = Some(NaiveDate::from_ymd_opt(2026, 5, 20).expect("valid date"));
        assert!(!compute_is_overdue(due_date, None, "Done", now));
    }

    #[test]
    fn overdue_true_for_past_due_datetime() {
        let now = NaiveDate::from_ymd_opt(2026, 5, 28)
            .expect("valid date")
            .and_hms_opt(10, 1, 0)
            .expect("valid time");
        let due_date = Some(NaiveDate::from_ymd_opt(2026, 5, 28).expect("valid date"));
        let due_time = Some(NaiveTime::from_hms_opt(10, 0, 0).expect("valid time"));
        assert!(compute_is_overdue(due_date, due_time, "In Progress", now));
    }

    #[test]
    fn overdue_false_before_end_of_due_day_when_no_time() {
        let now = NaiveDate::from_ymd_opt(2026, 5, 28)
            .expect("valid date")
            .and_hms_opt(23, 59, 59)
            .expect("valid time");
        let due_date = Some(NaiveDate::from_ymd_opt(2026, 5, 28).expect("valid date"));
        assert!(!compute_is_overdue(due_date, None, "Pending", now));
    }

    #[test]
    fn overdue_true_next_day_when_no_time() {
        let now = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2026, 5, 29).expect("valid date"),
            NaiveTime::from_hms_opt(0, 0, 0).expect("valid time"),
        );
        let due_date = Some(NaiveDate::from_ymd_opt(2026, 5, 28).expect("valid date"));
        assert!(compute_is_overdue(due_date, None, "Pending", now));
    }
}
