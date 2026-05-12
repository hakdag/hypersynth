use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::Argon2;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use password_hash::rand_core::OsRng;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::types::{
    ApiErrorBody, CompanyRegistrationRequest, CompanyRegistrationResponse,
};

const MIN_PASSWORD_LEN: usize = 8;
const MIN_USERNAME_LEN: usize = 3;
const MAX_USERNAME_LEN: usize = 64;

pub async fn register_company(
    State(state): State<AppState>,
    Json(payload): Json<CompanyRegistrationRequest>,
) -> Result<(StatusCode, Json<CompanyRegistrationResponse>), (StatusCode, Json<ApiErrorBody>)> {
    let name = payload.name.trim();
    let company_email = payload.company_email.trim();
    let country = payload.country.trim();
    let timezone = payload.timezone.trim();
    let full_name = payload.full_name.trim();
    let email = payload.email.trim();
    let username = payload.username.trim();
    let password = payload.password.as_str();
    let password_confirmation = payload.password_confirmation.as_str();

    if name.is_empty() {
        return Err(bad_request("Company name is required."));
    }

    if company_email.is_empty() {
        return Err(bad_request("Company email is required."));
    }

    if !email_contains_at_and_dot(company_email) {
        return Err(bad_request("Enter a valid company email address."));
    }

    if country.is_empty() {
        return Err(bad_request("Country is required."));
    }

    if timezone.is_empty() {
        return Err(bad_request("Timezone is required."));
    }

    if full_name.is_empty() {
        return Err(bad_request("Full name is required."));
    }

    if email.is_empty() {
        return Err(bad_request("Email is required."));
    }

    if !email_contains_at_and_dot(email) {
        return Err(bad_request("Enter a valid email address."));
    }

    if username.is_empty() {
        return Err(bad_request("Username is required."));
    }

    if !username_is_valid(username) {
        return Err(bad_request(
            "Username must be 3–64 characters and may only contain letters, numbers, underscores, dots, and hyphens.",
        ));
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

    if password != password_confirmation {
        return Err(bad_request("Password and confirmation do not match."));
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| internal_error())?;

    let hash_str = password_hash.to_string();

    let mut tx = state.pool.begin().await.map_err(|_| internal_error())?;

    let company_id = match sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO companies (name, company_email, country, timezone, status)
        VALUES ($1, lower(trim($2)), $3, $4, 'active')
        RETURNING id
        "#,
    )
    .bind(name)
    .bind(company_email)
    .bind(country)
    .bind(timezone)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(id) => id,
        Err(e) => {
            if let Some(db) = e.as_database_error() {
                if db.code().as_deref() == Some("23505") {
                    return Err(conflict_for_constraint(db.constraint()));
                }
            }
            return Err(internal_error());
        }
    };

    let user_id = match sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO users (
            fullname,
            email,
            username,
            password_hash,
            account_type,
            role,
            status
        )
        VALUES ($1, lower(trim($2)), lower(trim($3)), $4, 'company', 'company_admin', 'active')
        RETURNING id
        "#,
    )
    .bind(full_name)
    .bind(email)
    .bind(username)
    .bind(&hash_str)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(id) => id,
        Err(e) => {
            if let Some(db) = e.as_database_error() {
                if db.code().as_deref() == Some("23505") {
                    return Err(conflict_for_constraint(db.constraint()));
                }
            }
            return Err(internal_error());
        }
    };

    sqlx::query(
        r#"
        INSERT INTO company_users (company_id, user_id)
        VALUES ($1, $2)
        "#,
    )
    .bind(company_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| internal_error())?;

    tx.commit().await.map_err(|_| internal_error())?;

    Ok((
        StatusCode::CREATED,
        Json(CompanyRegistrationResponse {
            user_id,
            company_id,
            message: "Your company account has been created.".into(),
        }),
    ))
}

fn username_is_valid(username: &str) -> bool {
    let len = username.chars().count();
    if len < MIN_USERNAME_LEN || len > MAX_USERNAME_LEN {
        return false;
    }
    username
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' || ch == '-')
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

fn conflict_for_constraint(constraint: Option<&str>) -> (StatusCode, Json<ApiErrorBody>) {
    let message = match constraint {
        Some("idx_companies_company_email_lower") => {
            "A company with this email already exists.".into()
        }
        Some("idx_users_email_lower") => "An account with this email already exists.".into(),
        Some("idx_users_username_lower") => "This username is already taken.".into(),
        _ => "A record with these details already exists.".into(),
    };
    (
        StatusCode::CONFLICT,
        Json(ApiErrorBody { message }),
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
