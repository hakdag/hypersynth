use std::path::Path;
use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use chrono::{Duration as ChronoDuration, Utc};
use sqlx::PgConnection;
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::app_state::AppState;
use crate::auth_route::require_system_admin;
use crate::configs::SmtpConfig;
use crate::tx_extractor::missing_tx_error;
use crate::types::{
    AdminSystemHealthResponse, ApiErrorBody, HealthIndicator, HealthIndicatorStatus, Tx,
};

const AI_ERROR_RATE_WINDOW_HOURS: i64 = 24;
const AI_ERROR_RATE_DEGRADED_THRESHOLD: f64 = 0.10;

pub async fn get_admin_health(
    State(state): State<AppState>,
    tx: Tx,
    jar: CookieJar,
) -> Result<Json<AdminSystemHealthResponse>, (StatusCode, Json<ApiErrorBody>)> {
    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;
    let _admin_email = require_system_admin(conn, &jar).await?;

    let application = HealthIndicator {
        status: HealthIndicatorStatus::Healthy,
        summary: "Application is running.".into(),
        detail: None,
    };

    let database = check_database(conn).await;
    let background_jobs = HealthIndicator {
        status: HealthIndicatorStatus::NotConfigured,
        summary: "No background job runner configured.".into(),
        detail: None,
    };
    let ai_provider_error_rate = check_ai_error_rate(conn).await;
    let email_delivery = check_email_delivery(&state).await;
    let storage = check_storage(&state.document_upload_dir).await;

    Ok(Json(AdminSystemHealthResponse {
        application,
        database,
        background_jobs,
        ai_provider_error_rate,
        email_delivery,
        storage,
    }))
}

async fn check_database(conn: &mut PgConnection) -> HealthIndicator {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&mut *conn)
        .await
    {
        Ok(_) => HealthIndicator {
            status: HealthIndicatorStatus::Healthy,
            summary: "Database is reachable.".into(),
            detail: None,
        },
        Err(_) => HealthIndicator {
            status: HealthIndicatorStatus::Unavailable,
            summary: "Database is not reachable.".into(),
            detail: None,
        },
    }
}

async fn check_ai_error_rate(conn: &mut PgConnection) -> HealthIndicator {
    let from = Utc::now() - ChronoDuration::hours(AI_ERROR_RATE_WINDOW_HOURS);
    let to = Utc::now();

    let row: Option<(i64, i64)> = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)::bigint AS request_count,
            COUNT(*) FILTER (WHERE status = 'failed')::bigint AS failure_count
        FROM ai_usage
        WHERE created_at >= $1 AND created_at < $2
        "#,
    )
    .bind(from)
    .bind(to)
    .fetch_optional(&mut *conn)
    .await
    .ok()
    .flatten();

    let Some((request_count, failure_count)) = row else {
        return HealthIndicator {
            status: HealthIndicatorStatus::Unavailable,
            summary: "Could not read AI usage statistics.".into(),
            detail: None,
        };
    };

    if request_count == 0 {
        return HealthIndicator {
            status: HealthIndicatorStatus::Healthy,
            summary: "No AI requests in the last 24 hours.".into(),
            detail: Some("Error rate is not applicable without recent requests.".into()),
        };
    }

    let rate = failure_count as f64 / request_count as f64;
    let pct = (rate * 100.0).round();
    let detail = Some(format!(
        "{failure_count} failed of {request_count} requests in the last 24 hours ({pct}% failure rate)."
    ));

    if rate >= AI_ERROR_RATE_DEGRADED_THRESHOLD {
        HealthIndicator {
            status: HealthIndicatorStatus::Degraded,
            summary: format!("AI provider error rate is elevated ({pct}%)."),
            detail,
        }
    } else {
        HealthIndicator {
            status: HealthIndicatorStatus::Healthy,
            summary: format!("AI provider error rate is normal ({pct}%)."),
            detail,
        }
    }
}

async fn check_email_delivery(_state: &AppState) -> HealthIndicator {
    let cfg = match SmtpConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            return HealthIndicator {
                status: HealthIndicatorStatus::Unavailable,
                summary: "Email delivery is not configured.".into(),
                detail: Some(e),
            };
        }
    };

    let connect_result = timeout(
        Duration::from_secs(3),
        TcpStream::connect((cfg.host.as_str(), cfg.port)),
    )
    .await;

    match connect_result {
        Ok(Ok(_)) => HealthIndicator {
            status: HealthIndicatorStatus::Healthy,
            summary: "SMTP relay is reachable.".into(),
            detail: None,
        },
        Ok(Err(_)) => HealthIndicator {
            status: HealthIndicatorStatus::Degraded,
            summary: "SMTP relay is configured but not reachable.".into(),
            detail: None,
        },
        Err(_) => HealthIndicator {
            status: HealthIndicatorStatus::Degraded,
            summary: "SMTP relay connection timed out.".into(),
            detail: None,
        },
    }
}

async fn check_storage(upload_dir: &str) -> HealthIndicator {
    let path = Path::new(upload_dir);

    if !path.exists() {
        if std::fs::create_dir_all(path).is_ok() {
            return HealthIndicator {
                status: HealthIndicatorStatus::Degraded,
                summary: "Document storage directory was created.".into(),
                detail: None,
            };
        }
        return HealthIndicator {
            status: HealthIndicatorStatus::Unavailable,
            summary: "Document storage is not accessible.".into(),
            detail: None,
        };
    }

    let (file_count, total_bytes) = walk_storage_stats(path);
    let writable = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(path.join(".health_write_probe"))
        .and_then(|f| {
            drop(f);
            std::fs::remove_file(path.join(".health_write_probe"))
        })
        .is_ok();

    if !writable {
        return HealthIndicator {
            status: HealthIndicatorStatus::Unavailable,
            summary: "Document storage is not writable.".into(),
            detail: Some(format!("{file_count} files, {total_bytes} bytes stored.")),
        };
    }

    HealthIndicator {
        status: HealthIndicatorStatus::Healthy,
        summary: "Document storage is available.".into(),
        detail: Some(format!("{file_count} files, {total_bytes} bytes stored.")),
    }
}

fn walk_storage_stats(path: &Path) -> (u64, u64) {
    let mut file_count = 0u64;
    let mut total_bytes = 0u64;

    let entries = match std::fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return (0, 0),
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            let (sub_count, sub_bytes) = walk_storage_stats(&entry_path);
            file_count += sub_count;
            total_bytes += sub_bytes;
        } else if entry_path.is_file() {
            file_count += 1;
            if let Ok(meta) = entry.metadata() {
                total_bytes += meta.len();
            }
        }
    }

    (file_count, total_bytes)
}
