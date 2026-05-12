use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::Argon2;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use password_hash::rand_core::OsRng;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::types::{AccountType, ApiErrorBody, RegisterRequest, RegisterSuccessResponse};

const MIN_PASSWORD_LEN: usize = 8;

pub async fn register_user(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterSuccessResponse>), (StatusCode, Json<ApiErrorBody>)> {
    let fullname = payload.fullname.trim();
    let email = payload.email.trim();
    let password = payload.password.as_str();

    if fullname.is_empty() {
        return Err(bad_request("Full name is required."));
    }

    if email.is_empty() {
        return Err(bad_request("Email is required."));
    }

    if !email_contains_at_and_dot(email) {
        return Err(bad_request("Enter a valid email address."));
    }

    if password.len() < MIN_PASSWORD_LEN {
        return Err(bad_request(format!(
            "Password must be at least {} characters.",
            MIN_PASSWORD_LEN
        )));
    }

    if !password_has_letter_and_digit(password) {
        return Err(bad_request(
            "Password must include at least one letter and one number.",
        ));
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| internal_error())?;

    let hash_str = password_hash.to_string();
    let account_type = payload.account_type.as_db_value();

    if payload.account_type == AccountType::Company {
        return Err(bad_request(
            "Company registration is not supported on this endpoint. Use /api/v1/companies/register.",
        ));
    }

    let mut tx = state.pool.begin().await.map_err(|_| internal_error())?;

    let user_id = match sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO users (fullname, email, password_hash, account_type)
        VALUES ($1, lower(trim($2)), $3, $4)
        RETURNING id
        "#,
    )
    .bind(fullname)
    .bind(email)
    .bind(&hash_str)
    .bind(account_type)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(id) => id,
        Err(e) => {
            if let Some(db) = e.as_database_error() {
                if db.code().as_deref() == Some("23505") {
                    return Err((
                        StatusCode::CONFLICT,
                        Json(ApiErrorBody {
                            message: "An account with this email already exists.".into(),
                        }),
                    ));
                }
            }
            return Err(internal_error());
        }
    };

    tx.commit().await.map_err(|_| internal_error())?;

    Ok((
        StatusCode::CREATED,
        Json(RegisterSuccessResponse {
            id: user_id,
            message: "Your account has been created.".into(),
        }),
    ))
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

fn password_has_letter_and_digit(password: &str) -> bool {
    let mut letter = false;
    let mut digit = false;
    for ch in password.chars() {
        if ch.is_alphabetic() {
            letter = true;
        } else if ch.is_ascii_digit() {
            digit = true;
        }
        if letter && digit {
            return true;
        }
    }
    false
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
