use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::types::{AccountType, ApiErrorBody, RegisterRequest, RegisterSuccessResponse};
use crate::user_registration::{
    email_contains_at_and_dot, hash_password_argon2, password_policy_error,
};

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

    if let Some(msg) = password_policy_error(password) {
        return Err(bad_request(msg));
    }

    let hash_str = hash_password_argon2(password).map_err(|_| internal_error())?;
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
                        Json(ApiErrorBody::msg(
                            "An account with this email already exists.",
                        )),
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

fn bad_request(message: impl Into<String>) -> (StatusCode, Json<ApiErrorBody>) {
    (StatusCode::BAD_REQUEST, Json(ApiErrorBody::msg(message)))
}

fn internal_error() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiErrorBody::msg("Something went wrong. Please try again.")),
    )
}
