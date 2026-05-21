mod admin_ai_usage_route;
mod admin_company_route;
mod admin_user_route;
mod ai;
mod ai_provider_route;
mod ai_usage_service;
mod app_state;
mod auth_route;
mod authorization;
mod company_registration_route;
mod company_route;
mod configs;
mod crypto;
mod document_context_service;
mod email;
mod invitation_accept_route;
mod invitation_route;
mod invitation_token_service;
mod project_ai_settings_route;
mod project_ai_settings_service;
mod project_api_key_service;
mod project_membership_route;
mod project_route;
mod register_route;
mod runtime_decrypt_error;
mod tenant_scope_service;
mod types;
mod user_registration;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use admin_ai_usage_route::{
    admin_ai_usage_by_company, admin_ai_usage_by_provider_model, admin_ai_usage_by_user,
    admin_ai_usage_failures, admin_ai_usage_summary,
};
use admin_company_route::{get_admin_company, list_admin_companies, set_admin_company_status};
use admin_user_route::{
    get_admin_user, list_admin_users, reset_admin_user_access, set_admin_user_status,
};
use ai::{AiProviderRegistry, AnthropicProvider, OpenAiProvider};
use ai_provider_route::{list_provider_models, list_supported_providers};
use app_state::AppState;
use axum::extract::DefaultBodyLimit;
use axum::extract::State;
use axum::http::header::{ACCEPT, CONTENT_TYPE};
use axum::http::HeaderValue;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use company_registration_route::register_company;
use company_route::{get_current_company, list_company_users, update_current_company};
use configs::AppConfig;
use email::SmtpEmailSender;
use invitation_accept_route::{
    accept_invitation_confirm, accept_invitation_register, preview_invitation,
};
use invitation_route::{cancel_invitation, create_invitation, list_invitations};
use project_ai_settings_route::{
    get_project_ai_settings, list_project_ai_provider_models, update_project_ai_settings,
};
use project_membership_route::{add_project_member, list_project_members, remove_project_member};
use project_route::{
    accept_generated_tasks, create_feature, create_project, create_task, download_project_document,
    enhance_feature_requirements, enhance_project_requirements, generate_feature_tasks,
    get_project, get_project_feature, get_project_task, list_feature_tasks, list_project_documents,
    list_project_features, list_projects, update_project, update_project_feature,
    update_project_task, upload_project_documents,
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
        dotenvy::from_path(&path)
            .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
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

    let AppConfig {
        port,
        database_url,
        cors_origin,
        session_max_age_secs,
        document_upload_dir,
        api_key_encryption_key,
        anthropic_config,
        openai_config,
        invitation_config,
        smtp_config,
        system_admin_config,
    } = config;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&pool)
        .await?;

    let anthropic_http = reqwest::Client::builder()
        .timeout(Duration::from_secs(anthropic_config.timeout_secs))
        .build()?;
    let anthropic = AnthropicProvider::new(
        anthropic_http,
        anthropic_config.base_url,
        anthropic_config.model,
        anthropic_config.max_tokens,
    );
    let openai_http = reqwest::Client::builder()
        .timeout(Duration::from_secs(openai_config.timeout_secs))
        .build()?;
    let openai = OpenAiProvider::new(
        openai_http,
        openai_config.base_url,
        openai_config.default_model,
        openai_config.max_tokens,
    );
    let ai_providers = AiProviderRegistry::new(anthropic, openai);

    let cors = CorsLayer::new()
        .allow_origin(cors_origin.parse::<HeaderValue>()?)
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

    let smtp_sender =
        SmtpEmailSender::try_new(&smtp_config).map_err(|e| format!("SMTP setup: {e}"))?;
    let email_sender: Arc<dyn email::EmailSender + Send + Sync> = Arc::new(smtp_sender);

    let state = AppState {
        pool,
        session_max_age_secs,
        document_upload_dir,
        api_key_encryption_key,
        ai_providers,
        email_sender,
        invitation_config,
        system_admin: system_admin_config,
    };

    let app = Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/bootstrap", get(bootstrap))
        .route("/api/v1/register", post(register_user))
        .route("/api/v1/companies/register", post(register_company))
        .route(
            "/api/v1/company",
            get(get_current_company).patch(update_current_company),
        )
        .route("/api/v1/company/users", get(list_company_users))
        .route("/api/v1/login", post(auth_route::login))
        .route("/api/v1/logout", post(auth_route::logout))
        .route("/api/v1/me", get(auth_route::current_user))
        .route("/api/v1/ai/providers", get(list_supported_providers))
        .route(
            "/api/v1/ai/providers/{provider_id}/models",
            post(list_provider_models),
        )
        .route("/api/v1/admin/companies", get(list_admin_companies))
        .route(
            "/api/v1/admin/companies/{company_id}",
            get(get_admin_company),
        )
        .route(
            "/api/v1/admin/companies/{company_id}/status",
            post(set_admin_company_status),
        )
        .route("/api/v1/admin/users", get(list_admin_users))
        .route("/api/v1/admin/users/{user_id}", get(get_admin_user))
        .route(
            "/api/v1/admin/users/{user_id}/status",
            post(set_admin_user_status),
        )
        .route(
            "/api/v1/admin/users/{user_id}/reset-access",
            post(reset_admin_user_access),
        )
        .route(
            "/api/v1/admin/ai-usage/summary",
            get(admin_ai_usage_summary),
        )
        .route(
            "/api/v1/admin/ai-usage/by-company",
            get(admin_ai_usage_by_company),
        )
        .route(
            "/api/v1/admin/ai-usage/by-user",
            get(admin_ai_usage_by_user),
        )
        .route(
            "/api/v1/admin/ai-usage/by-provider-model",
            get(admin_ai_usage_by_provider_model),
        )
        .route(
            "/api/v1/admin/ai-usage/failures",
            get(admin_ai_usage_failures),
        )
        .route(
            "/api/v1/invitations",
            get(list_invitations).post(create_invitation),
        )
        .route(
            "/api/v1/invitations/{invitation_id}/cancel",
            post(cancel_invitation),
        )
        .route(
            "/api/v1/invitations/accept/preview",
            get(preview_invitation),
        )
        .route(
            "/api/v1/invitations/accept/register",
            post(accept_invitation_register),
        )
        .route(
            "/api/v1/invitations/accept/confirm",
            post(accept_invitation_confirm),
        )
        .route("/api/v1/projects", get(list_projects).post(create_project))
        .route(
            "/api/v1/projects/{project_id}",
            get(get_project).patch(update_project),
        )
        .route(
            "/api/v1/projects/{project_id}/members",
            get(list_project_members).post(add_project_member),
        )
        .route(
            "/api/v1/projects/{project_id}/members/{user_id}",
            delete(remove_project_member),
        )
        .route(
            "/api/v1/projects/{project_id}/ai/enhance-requirements",
            post(enhance_project_requirements),
        )
        .route(
            "/api/v1/projects/{project_id}/ai-settings",
            get(get_project_ai_settings).put(update_project_ai_settings),
        )
        .route(
            "/api/v1/projects/{project_id}/ai-settings/provider-models",
            post(list_project_ai_provider_models),
        )
        .route(
            "/api/v1/projects/{project_id}/features",
            get(list_project_features).post(create_feature),
        )
        .route(
            "/api/v1/projects/{project_id}/documents",
            get(list_project_documents)
                .post(upload_project_documents)
                .layer(DefaultBodyLimit::max(25 * 1024 * 1024)),
        )
        .route(
            "/api/v1/projects/{project_id}/documents/{document_id}/download",
            get(download_project_document),
        )
        .route(
            "/api/v1/projects/{project_id}/features/{feature_id}",
            get(get_project_feature).patch(update_project_feature),
        )
        .route(
            "/api/v1/projects/{project_id}/features/{feature_id}/ai/enhance-requirements",
            post(enhance_feature_requirements),
        )
        .route(
            "/api/v1/projects/{project_id}/features/{feature_id}/ai/generate-tasks",
            post(generate_feature_tasks),
        )
        .route(
            "/api/v1/projects/{project_id}/features/{feature_id}/ai/accept-tasks",
            post(accept_generated_tasks),
        )
        .route(
            "/api/v1/projects/{project_id}/features/{feature_id}/tasks",
            get(list_feature_tasks).post(create_task),
        )
        .route(
            "/api/v1/projects/{project_id}/features/{feature_id}/tasks/{task_id}",
            get(get_project_task).patch(update_project_task),
        )
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    eprintln!("listening on http://{}", addr);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

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
