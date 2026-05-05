use argon2::password_hash::{PasswordHash, PasswordVerifier};
use argon2::Argon2;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{Duration as ChronoDuration, Utc};
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use time::Duration as CookieDuration;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::types::{ApiErrorBody, CurrentUserBody, LoginRequest};

const SESSION_COOKIE: &str = "hypersynth_session";
const GENERIC_AUTH_FAILURE: &str = "Invalid email or password.";

pub(crate) fn has_session_cookie(jar: &CookieJar) -> bool {
    jar.get(SESSION_COOKIE).is_some()
}

#[derive(sqlx::FromRow)]
struct UserAuthRow {
    id: Uuid,
    fullname: String,
    email: String,
    password_hash: String,
}

pub async fn login(
    State(state): State<AppState>,
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

    let row = sqlx::query_as::<_, UserAuthRow>(
        r#"SELECT id, fullname, email, password_hash FROM users WHERE email = lower(trim($1))"#,
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

    let mut raw = [0u8; 32];
    OsRng.fill_bytes(&mut raw);
    let token_hash = hash_session_token(&raw);
    let expires_at = Utc::now() + ChronoDuration::seconds(state.session_max_age_secs);

    sqlx::query(
        r#"INSERT INTO sessions (user_id, token_hash, expires_at) VALUES ($1, $2, $3)"#,
    )
    .bind(user.id)
    .bind(&token_hash)
    .bind(expires_at)
    .execute(&state.pool)
    .await
    .map_err(|_| internal_error())?;

    let token_cookie = hex::encode(raw);
    let cookie = Cookie::build((SESSION_COOKIE, token_cookie))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(CookieDuration::seconds(state.session_max_age_secs))
        .build();

    let jar = jar.add(cookie);
    let body = CurrentUserBody {
        id: user.id,
        fullname: user.fullname,
        email: user.email,
    };
    Ok((jar, Json(body)))
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
) -> Result<Json<CurrentUserBody>, (StatusCode, Json<ApiErrorBody>)> {
    match resolve_current_user(&state.pool, &jar).await {
        Some(u) => Ok(Json(u)),
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiErrorBody {
                message: "You need to sign in to continue.".into(),
            }),
        )),
    }
}

pub(crate) async fn require_authenticated_user(
    pool: &PgPool,
    jar: &CookieJar,
) -> Result<CurrentUserBody, (StatusCode, Json<ApiErrorBody>)> {
    match resolve_current_user(pool, jar).await {
        Some(u) => Ok(u),
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiErrorBody {
                message: "You need to sign in to continue.".into(),
            }),
        )),
    }
}

async fn resolve_current_user(pool: &PgPool, jar: &CookieJar) -> Option<CurrentUserBody> {
    let cookie = jar.get(SESSION_COOKIE)?;
    let bytes = hex::decode(cookie.value()).ok()?;
    let raw = <[u8; 32]>::try_from(bytes.as_slice()).ok()?;
    let token_hash = hash_session_token(&raw);
    sqlx::query_as::<_, CurrentUserBody>(
        r#"SELECT u.id, u.fullname, u.email FROM sessions s
           INNER JOIN users u ON u.id = s.user_id
           WHERE s.token_hash = $1 AND s.expires_at > now()"#,
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await
    .ok()?
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

fn email_contains_at_and_dot(email: &str) -> bool {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let domain = parts[1];
    domain.contains('.')
        && !parts[0].is_empty()
        && !domain.starts_with('.')
        && !domain.ends_with('.')
}

fn unauthorized_auth() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(ApiErrorBody {
            message: GENERIC_AUTH_FAILURE.into(),
        }),
    )
}

fn bad_request(message: impl Into<String>) -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ApiErrorBody {
            message: message.into(),
        }),
    )
}

fn internal_error() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiErrorBody {
            message: "Something went wrong. Please try again.".into(),
        }),
    )
}
