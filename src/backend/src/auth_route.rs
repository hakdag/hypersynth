use std::net::SocketAddr;

use argon2::password_hash::{PasswordHash, PasswordVerifier};
use argon2::Argon2;
use axum::extract::ConnectInfo;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{Duration as ChronoDuration, Utc};
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use time::Duration as CookieDuration;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::types::{
    AccountType, ApiErrorBody, CompanyRole, CompanyStatus, CurrentUserBody, LoginRequest,
    SessionPrincipal, SessionUser, UserStatus, ERROR_CODE_COMPANY_DISABLED,
    ERROR_CODE_USER_DISABLED,
};
use crate::user_registration::email_contains_at_and_dot;

const SESSION_COOKIE: &str = "hypersynth_session";
const GENERIC_AUTH_FAILURE: &str = "Invalid email or password.";
const SYSTEM_ADMIN_DISPLAY_NAME: &str = "System Admin";

pub(crate) fn has_session_cookie(jar: &CookieJar) -> bool {
    jar.get(SESSION_COOKIE).is_some()
}

#[derive(sqlx::FromRow)]
struct UserAuthRow {
    id: Uuid,
    password_hash: String,
    account_type: String,
    status: String,
}

#[derive(sqlx::FromRow)]
struct SessionUserRow {
    id: Uuid,
    fullname: String,
    email: String,
    avatar_url: Option<String>,
    account_type: String,
    role: Option<String>,
    company_id: Option<Uuid>,
}

#[derive(sqlx::FromRow)]
struct SessionResolveRow {
    is_system_admin: bool,
    system_admin_email: Option<String>,
    id: Option<Uuid>,
    fullname: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
    account_type: Option<String>,
    role: Option<String>,
    company_id: Option<Uuid>,
    company_status: Option<String>,
    user_status: Option<String>,
}

pub async fn login(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiErrorBody>)> {
    let email = payload.email.trim();
    let password = payload.password.as_str();

    if email.is_empty() {
        return Err(bad_request("Email is required."));
    }
    if password.is_empty() {
        return Err(bad_request("Password is required."));
    }
    if !email_contains_at_and_dot(email) {
        return Err(bad_request("Enter a valid email address."));
    }

    let normalized_email = email.to_lowercase();

    if state.system_admin.enabled && normalized_email == state.system_admin.email {
        let ip = client_ip(&peer, &headers);
        let ua = user_agent(&headers);

        if !verify_password_hash(state.system_admin.password_hash.as_str(), password) {
            // Temporary until SF-24 audit logging persists these events.
            log_system_admin_attempt(&normalized_email, &ip, &ua, "failure");
            return Err(unauthorized_auth());
        }

        log_system_admin_attempt(&normalized_email, &ip, &ua, "success");

        let (jar, body) = establish_session_for_system_admin(
            &state.pool,
            state.session_max_age_secs,
            jar,
            &normalized_email,
        )
        .await?;
        return Ok((jar, Json(body)));
    }

    let row = sqlx::query_as::<_, UserAuthRow>(
        r#"
        SELECT
            u.id,
            u.password_hash,
            u.account_type,
            u.status
        FROM users u
        WHERE u.email = lower(trim($1))
        "#,
    )
    .bind(email)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| internal_error())?;

    let user = match row {
        Some(u) => u,
        None => return Err(unauthorized_auth()),
    };

    if !verify_password_hash(user.password_hash.as_str(), password) {
        return Err(unauthorized_auth());
    }

    if user.status == UserStatus::Disabled.as_db_value() {
        return Err(unauthorized_auth());
    }

    if user.account_type == "company" && user_company_is_disabled(&state.pool, user.id).await? {
        return Err(company_disabled_error());
    }

    let (jar, body) =
        establish_session_for_user(&state.pool, state.session_max_age_secs, jar, user.id).await?;
    Ok((jar, Json(body)))
}

/// Creates a new session row and session cookie for the given user (e.g. after login or invitation acceptance).
pub async fn establish_session_for_user(
    pool: &PgPool,
    session_max_age_secs: i64,
    jar: CookieJar,
    user_id: Uuid,
) -> Result<(CookieJar, CurrentUserBody), (StatusCode, Json<ApiErrorBody>)> {
    let row = sqlx::query_as::<_, SessionUserRow>(
        r#"
        SELECT
            u.id,
            u.fullname,
            u.email,
            u.avatar_url,
            u.account_type,
            u.role,
            cu.company_id
        FROM users u
        LEFT JOIN company_users cu ON cu.user_id = u.id
        WHERE u.id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| internal_error())?
    .ok_or_else(internal_error)?;

    let mut raw = [0u8; 32];
    OsRng.fill_bytes(&mut raw);
    let token_hash = hash_session_token(&raw);
    let expires_at = Utc::now() + ChronoDuration::seconds(session_max_age_secs);

    sqlx::query(
        r#"
        INSERT INTO sessions (user_id, token_hash, expires_at, is_system_admin)
        VALUES ($1, $2, $3, false)
        "#,
    )
    .bind(row.id)
    .bind(&token_hash)
    .bind(expires_at)
    .execute(pool)
    .await
    .map_err(|_| internal_error())?;

    let account_type =
        AccountType::from_db_value(row.account_type.as_str()).ok_or_else(internal_error)?;
    let role = decode_role(row.role.as_deref())?;

    let user = SessionUser {
        id: row.id,
        fullname: row.fullname,
        email: row.email,
        avatar_url: row.avatar_url,
        account_type,
        role,
        company_id: row.company_id,
    };

    let jar = jar.add(session_cookie(&raw, session_max_age_secs));
    Ok((jar, principal_to_body(SessionPrincipal::User(user))))
}

async fn establish_session_for_system_admin(
    pool: &PgPool,
    session_max_age_secs: i64,
    jar: CookieJar,
    email: &str,
) -> Result<(CookieJar, CurrentUserBody), (StatusCode, Json<ApiErrorBody>)> {
    let mut raw = [0u8; 32];
    OsRng.fill_bytes(&mut raw);
    let token_hash = hash_session_token(&raw);
    let expires_at = Utc::now() + ChronoDuration::seconds(session_max_age_secs);

    sqlx::query(
        r#"
        INSERT INTO sessions (
            user_id,
            token_hash,
            expires_at,
            is_system_admin,
            system_admin_email
        )
        VALUES (NULL, $1, $2, true, $3)
        "#,
    )
    .bind(&token_hash)
    .bind(expires_at)
    .bind(email)
    .execute(pool)
    .await
    .map_err(|_| internal_error())?;

    let jar = jar.add(session_cookie(&raw, session_max_age_secs));
    Ok((
        jar,
        principal_to_body(SessionPrincipal::SystemAdmin {
            email: email.to_string(),
        }),
    ))
}

pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        if let Ok(bytes) = hex::decode(cookie.value()) {
            if let Ok(raw) = <[u8; 32]>::try_from(bytes.as_slice()) {
                let token_hash = hash_session_token(&raw);
                let _ = sqlx::query("DELETE FROM sessions WHERE token_hash = $1")
                    .bind(&token_hash)
                    .execute(&state.pool)
                    .await;
            }
        }
    }
    let expired = Cookie::build((SESSION_COOKIE, ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(CookieDuration::seconds(0))
        .build();
    jar.add(expired)
}

pub async fn current_user(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Response, (StatusCode, Json<ApiErrorBody>)> {
    match resolve_current_principal(&state.pool, &jar).await {
        Ok(Some(principal)) => Ok(Json(principal_to_body(principal)).into_response()),
        Ok(None) => Err(unauthenticated()),
        Err(err) if is_company_disabled_error(&err) || is_user_disabled_error(&err) => {
            Ok((clear_session_jar(jar), err).into_response())
        }
        Err(err) => Err(err),
    }
}

pub(crate) async fn require_authenticated_user(
    pool: &PgPool,
    jar: &CookieJar,
) -> Result<SessionUser, (StatusCode, Json<ApiErrorBody>)> {
    match resolve_current_principal(pool, jar).await? {
        Some(SessionPrincipal::User(user)) => Ok(user),
        Some(SessionPrincipal::SystemAdmin { .. }) => Err(admin_forbidden()),
        None => Err(unauthenticated()),
    }
}

pub(crate) async fn require_system_admin(
    pool: &PgPool,
    jar: &CookieJar,
) -> Result<String, (StatusCode, Json<ApiErrorBody>)> {
    match resolve_current_principal(pool, jar).await? {
        Some(SessionPrincipal::SystemAdmin { email }) => Ok(email),
        Some(SessionPrincipal::User(_)) => Err(non_admin_forbidden()),
        None => Err(unauthenticated()),
    }
}

async fn resolve_current_principal(
    pool: &PgPool,
    jar: &CookieJar,
) -> Result<Option<SessionPrincipal>, (StatusCode, Json<ApiErrorBody>)> {
    let Some(cookie) = jar.get(SESSION_COOKIE) else {
        return Ok(None);
    };
    let Ok(bytes) = hex::decode(cookie.value()) else {
        return Ok(None);
    };
    let Ok(raw) = <[u8; 32]>::try_from(bytes.as_slice()) else {
        return Ok(None);
    };
    let token_hash = hash_session_token(&raw);

    let row = sqlx::query_as::<_, SessionResolveRow>(
        r#"
        SELECT
            s.is_system_admin,
            s.system_admin_email,
            u.id,
            u.fullname,
            u.email,
            u.avatar_url,
            u.account_type,
            u.role,
            cu.company_id,
            c.status AS company_status,
            u.status AS user_status
        FROM sessions s
        LEFT JOIN users u ON u.id = s.user_id
        LEFT JOIN company_users cu ON cu.user_id = u.id
        LEFT JOIN companies c ON c.id = cu.company_id
        WHERE s.token_hash = $1 AND s.expires_at > now()
        "#,
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await
    .map_err(|_| internal_error())?;

    let Some(row) = row else {
        return Ok(None);
    };

    if row.is_system_admin {
        let Some(email) = row.system_admin_email else {
            return Err(internal_error());
        };
        return Ok(Some(SessionPrincipal::SystemAdmin { email }));
    }

    let id = row.id.ok_or_else(internal_error)?;
    let fullname = row.fullname.ok_or_else(internal_error)?;
    let email = row.email.ok_or_else(internal_error)?;
    let account_type = row
        .account_type
        .as_deref()
        .and_then(AccountType::from_db_value)
        .ok_or_else(internal_error)?;
    let role = decode_role(row.role.as_deref())?;

    if account_type == AccountType::Company
        && row.company_status.as_deref() == Some(CompanyStatus::Disabled.as_db_value())
    {
        revoke_session_by_token_hash(pool, &token_hash).await?;
        return Err(company_disabled_error());
    }

    if row.user_status.as_deref() == Some(UserStatus::Disabled.as_db_value()) {
        revoke_session_by_token_hash(pool, &token_hash).await?;
        return Err(user_disabled_error());
    }

    Ok(Some(SessionPrincipal::User(SessionUser {
        id,
        fullname,
        email,
        avatar_url: row.avatar_url,
        account_type,
        role,
        company_id: row.company_id,
    })))
}

async fn user_company_is_disabled(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<bool, (StatusCode, Json<ApiErrorBody>)> {
    let status: Option<String> = sqlx::query_scalar(
        r#"
        SELECT c.status
        FROM company_users cu
        INNER JOIN companies c ON c.id = cu.company_id
        WHERE cu.user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| internal_error())?;

    Ok(status.as_deref() == Some(CompanyStatus::Disabled.as_db_value()))
}

async fn revoke_session_by_token_hash(
    pool: &PgPool,
    token_hash: &str,
) -> Result<(), (StatusCode, Json<ApiErrorBody>)> {
    sqlx::query("DELETE FROM sessions WHERE token_hash = $1")
        .bind(token_hash)
        .execute(pool)
        .await
        .map_err(|_| internal_error())?;
    Ok(())
}

fn clear_session_jar(jar: CookieJar) -> CookieJar {
    let expired = Cookie::build((SESSION_COOKIE, ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(CookieDuration::seconds(0))
        .build();
    jar.add(expired)
}

fn is_company_disabled_error(err: &(StatusCode, Json<ApiErrorBody>)) -> bool {
    err.0 == StatusCode::FORBIDDEN && err.1.code.as_deref() == Some(ERROR_CODE_COMPANY_DISABLED)
}

fn is_user_disabled_error(err: &(StatusCode, Json<ApiErrorBody>)) -> bool {
    err.0 == StatusCode::FORBIDDEN && err.1.code.as_deref() == Some(ERROR_CODE_USER_DISABLED)
}

fn company_disabled_error() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::FORBIDDEN,
        Json(ApiErrorBody::company_disabled()),
    )
}

fn user_disabled_error() -> (StatusCode, Json<ApiErrorBody>) {
    (StatusCode::FORBIDDEN, Json(ApiErrorBody::user_disabled()))
}

fn principal_to_body(principal: SessionPrincipal) -> CurrentUserBody {
    match principal {
        SessionPrincipal::User(user) => CurrentUserBody {
            id: user.id,
            fullname: user.fullname,
            email: user.email,
            avatar_url: user.avatar_url,
            account_type: user.account_type,
            role: user.role,
            company_id: user.company_id,
        },
        SessionPrincipal::SystemAdmin { email } => CurrentUserBody {
            id: Uuid::nil(),
            fullname: SYSTEM_ADMIN_DISPLAY_NAME.into(),
            email,
            avatar_url: None,
            account_type: AccountType::SystemAdmin,
            role: None,
            company_id: None,
        },
    }
}

fn session_cookie(raw: &[u8; 32], session_max_age_secs: i64) -> Cookie<'static> {
    let token_cookie = hex::encode(raw);
    Cookie::build((SESSION_COOKIE, token_cookie))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(CookieDuration::seconds(session_max_age_secs))
        .build()
}

fn decode_role(
    value: Option<&str>,
) -> Result<Option<CompanyRole>, (StatusCode, Json<ApiErrorBody>)> {
    match value {
        Some(raw) => match CompanyRole::from_db_value(raw) {
            Some(role) => Ok(Some(role)),
            None => Err(internal_error()),
        },
        None => Ok(None),
    }
}

fn hash_session_token(raw: &[u8; 32]) -> String {
    hex::encode(Sha256::digest(raw))
}

fn verify_password_hash(hash: &str, password: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

fn client_ip(peer: &SocketAddr, headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| peer.ip().to_string())
}

fn user_agent(headers: &HeaderMap) -> String {
    headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string()
}

/// Temporary until SF-24 audit logging persists System Admin login attempts.
fn log_system_admin_attempt(email: &str, ip: &str, ua: &str, outcome: &str) {
    if outcome == "success" {
        tracing::info!(
            target: "system_admin_auth",
            email = %email,
            ip = %ip,
            ua = %ua,
            outcome = %outcome,
            "system admin login attempt"
        );
    } else {
        tracing::warn!(
            target: "system_admin_auth",
            email = %email,
            ip = %ip,
            ua = %ua,
            outcome = %outcome,
            "system admin login attempt"
        );
    }
}

fn unauthorized_auth() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(ApiErrorBody {
            message: GENERIC_AUTH_FAILURE.into(),
            ..Default::default()
        }),
    )
}

fn unauthenticated() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(ApiErrorBody {
            message: "You need to sign in to continue.".into(),
            ..Default::default()
        }),
    )
}

fn admin_forbidden() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::FORBIDDEN,
        Json(ApiErrorBody {
            message: "This action is not available to system administrators.".into(),
            ..Default::default()
        }),
    )
}

fn non_admin_forbidden() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::FORBIDDEN,
        Json(ApiErrorBody {
            message: "You do not have permission to access this resource.".into(),
            ..Default::default()
        }),
    )
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
