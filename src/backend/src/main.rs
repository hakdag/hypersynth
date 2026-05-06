mod app_state;
mod auth_route;
mod configs;
mod project_route;
mod register_route;
mod types;

use std::path::Path;

use app_state::AppState;
use axum::extract::State;
use axum::http::HeaderValue;
use axum::http::header::{ACCEPT, CONTENT_TYPE};
use axum::routing::{get, post};
use axum::{Json, Router};
use configs::AppConfig;
use project_route::{
    create_feature, create_project, create_task, get_project, get_project_feature, get_project_task,
    list_feature_tasks, list_project_features, list_projects, update_project,
    update_project_feature,
};
use register_route::register_user;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use types::{BootstrapResponse, HealthResponse};

fn init_http_logging() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        tracing_subscriber::EnvFilter::new("info,tower_http=debug,hypersynth_api=debug")
    });
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .init();
}

/// Loads `src/.env` relative to this crate (`src/backend` → parent `src` + `.env`).
fn load_src_env() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(".env");
    if path.exists() {
        dotenvy::from_path(&path).map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
        eprintln!("loaded environment from {}", path.display());
    } else {
        eprintln!(
            "note: {} not found; using process environment only",
            path.display()
        );
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_http_logging();

    load_src_env()?;
    let config = AppConfig::from_env().map_err(|e| format!("configuration error: {}", e))?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&pool)
        .await?;

    let cors = CorsLayer::new()
        .allow_origin(config.cors_origin.parse::<HeaderValue>()?)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::PATCH,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([CONTENT_TYPE, ACCEPT])
        .allow_credentials(true);

    let state = AppState {
        pool,
        session_max_age_secs: config.session_max_age_secs,
    };

    let app = Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/bootstrap", get(bootstrap))
        .route("/api/v1/register", post(register_user))
        .route("/api/v1/login", post(auth_route::login))
        .route("/api/v1/logout", post(auth_route::logout))
        .route("/api/v1/me", get(auth_route::current_user))
        .route("/api/v1/projects", get(list_projects).post(create_project))
        .route(
            "/api/v1/projects/{project_id}",
            get(get_project).patch(update_project),
        )
        .route(
            "/api/v1/projects/{project_id}/features",
            get(list_project_features).post(create_feature),
        )
        .route(
            "/api/v1/projects/{project_id}/features/{feature_id}",
            get(get_project_feature).patch(update_project_feature),
        )
        .route(
            "/api/v1/projects/{project_id}/features/{feature_id}/tasks",
            get(list_feature_tasks).post(create_task),
        )
        .route(
            "/api/v1/projects/{project_id}/features/{feature_id}/tasks/{task_id}",
            get(get_project_task),
        )
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    eprintln!("listening on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let database = match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await
    {
        Ok(_) => "ok",
        Err(_) => "unavailable",
    };

    Json(HealthResponse {
        status: "ok",
        database,
    })
}

async fn bootstrap() -> Json<BootstrapResponse> {
    Json(BootstrapResponse {
        app_name: "HyperSynth",
        status_labels: ["Pending", "In Progress", "Done"],
    })
}
