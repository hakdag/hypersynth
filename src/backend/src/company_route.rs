use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth_route::require_authenticated_user;
use crate::authorization;
use crate::types::{
    AccountType, ApiErrorBody, CompanyResponse, CompanyRole, CompanyUserResponse,
    UpdateCompanyRequest,
};
use crate::user_registration::email_contains_at_and_dot;

pub async fn list_company_users(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<Vec<CompanyUserResponse>>, (StatusCode, Json<ApiErrorBody>)> {
    let user = require_authenticated_user(&state.pool, &jar).await?;
    if user.account_type != AccountType::Company {
        return Err(authorization::forbidden(
            "You do not have permission to perform this action.",
        ));
    }
    let company_id = user.company_id.ok_or_else(|| {
        authorization::forbidden("Company membership is required for this action.")
    })?;

    let rows = sqlx::query_as::<_, (Uuid, String, String, Option<String>)>(
        r#"
        SELECT u.id, u.fullname, u.email, u.role
        FROM users u
        INNER JOIN company_users cu ON cu.user_id = u.id
        WHERE cu.company_id = $1
        ORDER BY lower(u.fullname) ASC
        "#,
    )
    .bind(company_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| internal_error())?;

    let mut out = Vec::with_capacity(rows.len());
    for (id, fullname, email, role_raw) in rows {
        let role = role_raw.as_deref().and_then(CompanyRole::from_db_value);
        out.push(CompanyUserResponse {
            id,
            fullname,
            email,
            role,
        });
    }

    Ok(Json(out))
}

pub async fn get_current_company(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<CompanyResponse>, (StatusCode, Json<ApiErrorBody>)> {
    let user = require_authenticated_user(&state.pool, &jar).await?;
    let company = fetch_company_for_user(&state.pool, user.id).await?;
    Ok(Json(company))
}

pub async fn update_current_company(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<UpdateCompanyRequest>,
) -> Result<Json<CompanyResponse>, (StatusCode, Json<ApiErrorBody>)> {
    let user = require_authenticated_user(&state.pool, &jar).await?;
    authorization::require_company_role(&user, authorization::MANAGE_COMPANY_PROFILE).await?;
    let company = fetch_company_for_user(&state.pool, user.id).await?;

    let name = payload.name.trim();
    let company_email = payload.company_email.trim();
    let country = payload.country.trim();
    let timezone = payload.timezone.trim();

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

    let legal_name = optional_trimmed(payload.legal_name);
    let website = optional_trimmed(payload.website);
    let industry = optional_trimmed(payload.industry);
    let company_size = optional_trimmed(payload.company_size);
    let phone = optional_trimmed(payload.phone);
    let billing_email = optional_trimmed(payload.billing_email);
    let address = optional_trimmed(payload.address);
    let tax_vat_number = optional_trimmed(payload.tax_vat_number);

    if let Some(ref email) = billing_email {
        if !email_contains_at_and_dot(email) {
            return Err(bad_request("Enter a valid billing email address."));
        }
    }

    let updated = match sqlx::query_as::<_, CompanyResponse>(
        r#"
        UPDATE companies
        SET
            name = $2,
            company_email = lower(trim($3)),
            country = $4,
            timezone = $5,
            legal_name = $6,
            website = $7,
            industry = $8,
            company_size = $9,
            phone = $10,
            billing_email = $11,
            address = $12,
            tax_vat_number = $13,
            updated_at = now()
        WHERE id = $1
        RETURNING
            id,
            name,
            company_email,
            country,
            timezone,
            legal_name,
            website,
            industry,
            company_size,
            phone,
            billing_email,
            address,
            tax_vat_number,
            status,
            created_at,
            updated_at
        "#,
    )
    .bind(company.id)
    .bind(name)
    .bind(company_email)
    .bind(country)
    .bind(timezone)
    .bind(legal_name.as_deref())
    .bind(website.as_deref())
    .bind(industry.as_deref())
    .bind(company_size.as_deref())
    .bind(phone.as_deref())
    .bind(billing_email.as_deref())
    .bind(address.as_deref())
    .bind(tax_vat_number.as_deref())
    .fetch_one(&state.pool)
    .await
    {
        Ok(row) => row,
        Err(e) => {
            if let Some(db) = e.as_database_error() {
                if db.code().as_deref() == Some("23505") {
                    return Err(conflict_for_constraint(db.constraint()));
                }
            }
            return Err(internal_error());
        }
    };

    Ok(Json(updated))
}

async fn fetch_company_for_user(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> Result<CompanyResponse, (StatusCode, Json<ApiErrorBody>)> {
    sqlx::query_as::<_, CompanyResponse>(
        r#"
        SELECT
            c.id,
            c.name,
            c.company_email,
            c.country,
            c.timezone,
            c.legal_name,
            c.website,
            c.industry,
            c.company_size,
            c.phone,
            c.billing_email,
            c.address,
            c.tax_vat_number,
            c.status,
            c.created_at,
            c.updated_at
        FROM companies c
        INNER JOIN company_users cu ON cu.company_id = c.id
        WHERE cu.user_id = $1
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| internal_error())?
    .ok_or_else(|| not_found())
}

fn optional_trimmed(value: Option<String>) -> Option<String> {
    match value {
        Some(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        None => None,
    }
}

fn conflict_for_constraint(constraint: Option<&str>) -> (StatusCode, Json<ApiErrorBody>) {
    let message = match constraint {
        Some("idx_companies_company_email_lower") => {
            "A company with this email already exists.".into()
        }
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

fn not_found() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiErrorBody {
            message: "No company is associated with your account.".into(),
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
