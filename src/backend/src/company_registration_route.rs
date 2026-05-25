use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::tx_extractor::missing_tx_error;
use crate::types::{ApiErrorBody, CompanyRegistrationRequest, CompanyRegistrationResponse, Tx};
use crate::user_registration::{
    email_contains_at_and_dot, hash_password_argon2, password_policy_error, username_is_valid,
    USERNAME_VALIDATION_MESSAGE,
};

pub async fn register_company(
    tx: Tx,
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
        return Err(bad_request(USERNAME_VALIDATION_MESSAGE));
    }

    if let Some(msg) = password_policy_error(password) {
        return Err(bad_request(msg));
    }

    if password != password_confirmation {
        return Err(bad_request("Password and confirmation do not match."));
    }

    let hash_str = hash_password_argon2(password).map_err(|_| internal_error())?;

    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;

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
    .fetch_one(&mut **conn)
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
    .fetch_one(&mut **conn)
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
    .execute(&mut **conn)
    .await
    .map_err(|_| internal_error())?;

    Ok((
        StatusCode::CREATED,
        Json(CompanyRegistrationResponse {
            user_id,
            company_id,
            message: "Your company account has been created.".into(),
        }),
    ))
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
        Json(ApiErrorBody {
            message,
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
