use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use chrono::Utc;
use sqlx::{PgPool, Postgres, Transaction};
use tracing::warn;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth_route;
use crate::invitation_token_service::{decode_invitation_token_hex, hash_invitation_token};
use crate::user_registration::{
    hash_password_argon2, password_policy_error, username_is_valid, USERNAME_VALIDATION_MESSAGE,
};
use crate::types::{
    AcceptInvitationConfirmRequest, AcceptInvitationRegisterRequest, AccountType, ApiErrorBody,
    CompanyRole, CurrentUserBody, Invitation, InvitationAcceptPreviewQuery,
    InvitationPreviewResponse, InvitationStatus, ProjectMembershipRole,
};

#[derive(sqlx::FromRow)]
struct InvitationPreviewDbRow {
    id: Uuid,
    invited_email: String,
    invited_role: String,
    status: String,
    expires_at: chrono::DateTime<Utc>,
    company_name: String,
    project_name: Option<String>,
}

pub async fn preview_invitation(
    State(state): State<AppState>,
    Query(query): Query<InvitationAcceptPreviewQuery>,
) -> Result<Json<InvitationPreviewResponse>, (StatusCode, Json<ApiErrorBody>)> {
    let token_hex = query.token.trim();
    if token_hex.is_empty() {
        return Err(bad_request("Invitation token is required."));
    }

    let raw = match decode_invitation_token_hex(token_hex) {
        Some(b) => b,
        None => return Err(not_found_invitation()),
    };
    let token_hash = hash_invitation_token(&raw);

    let row = sqlx::query_as::<_, InvitationPreviewDbRow>(
        r#"
        SELECT
            i.id,
            i.invited_email,
            i.invited_role,
            i.status,
            i.expires_at,
            c.name AS company_name,
            p.name AS project_name
        FROM invitations i
        INNER JOIN companies c ON c.id = i.company_id
        LEFT JOIN projects p ON p.id = i.project_id
        WHERE i.invitation_token_hash = $1
        "#,
    )
    .bind(&token_hash)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| internal_error())?;

    let Some(row) = row else {
        return Err(not_found_invitation());
    };

    let status = InvitationStatus::from_db_value(row.status.as_str())
        .ok_or_else(|| internal_error())?;

    if status == InvitationStatus::Pending && row.expires_at < Utc::now() {
        let _ = sqlx::query(
            r#"
            UPDATE invitations
            SET status = $2
            WHERE id = $1 AND status = $3
            "#,
        )
        .bind(row.id)
        .bind(InvitationStatus::Expired.as_db_value())
        .bind(InvitationStatus::Pending.as_db_value())
        .execute(&state.pool)
        .await;

        return Err(gone_invitation(
            "This invitation has expired.",
            InvitationStatus::Expired,
        ));
    }

    if status != InvitationStatus::Pending {
        return Err(gone_invitation(
            invitation_inactive_message(status),
            status,
        ));
    }

    let invited_role =
        CompanyRole::from_db_value(row.invited_role.as_str()).ok_or_else(internal_error)?;

    let existing_user_present: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)",
    )
    .bind(&row.invited_email)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| internal_error())?;

    Ok(Json(InvitationPreviewResponse {
        company_name: row.company_name,
        project_name: row.project_name,
        invited_role,
        invited_email: row.invited_email,
        status,
        expires_at: row.expires_at,
        existing_user_present,
    }))
}

fn invitation_inactive_message(status: InvitationStatus) -> &'static str {
    match status {
        InvitationStatus::Accepted => "This invitation has already been accepted.",
        InvitationStatus::Cancelled => "This invitation was cancelled.",
        InvitationStatus::Expired => "This invitation has expired.",
        InvitationStatus::Pending => "This invitation is no longer valid.",
    }
}

pub async fn accept_invitation_register(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<AcceptInvitationRegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiErrorBody>)> {
    let token_hex = payload.token.trim();
    if token_hex.is_empty() {
        return Err(bad_request("Invitation token is required."));
    }

    let raw = match decode_invitation_token_hex(token_hex) {
        Some(b) => b,
        None => return Err(not_found_invitation()),
    };
    let token_hash = hash_invitation_token(&raw);

    let fullname = payload.fullname.trim();
    if fullname.is_empty() {
        return Err(bad_request("Full name is required."));
    }

    let username = payload.username.trim();
    if username.is_empty() {
        return Err(bad_request("Username is required."));
    }
    if !username_is_valid(username) {
        return Err(bad_request(USERNAME_VALIDATION_MESSAGE));
    }

    let password = payload.password.as_str();
    let password_confirmation = payload.password_confirmation.as_str();

    if let Some(msg) = password_policy_error(password) {
        return Err(bad_request(msg));
    }

    if password != password_confirmation {
        return Err(bad_request("Password and confirmation do not match."));
    }

    let timezone = payload
        .timezone
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let mut tx = state.pool.begin().await.map_err(|_| internal_error())?;

    let invitation = match lock_pending_invitation(&mut tx, &token_hash).await? {
        Some(inv) => inv,
        None => {
            tx.rollback().await.ok();
            return Err(not_found_invitation());
        }
    };

    if invitation.expires_at < Utc::now() {
        expire_invitation_if_pending(&mut tx, invitation.id).await?;
        tx.commit().await.map_err(|_| internal_error())?;
        return Err(gone_invitation(
            "This invitation has expired.",
            InvitationStatus::Expired,
        ));
    }

    let st = match InvitationStatus::from_db_value(invitation.status.as_str()) {
        Some(s) => s,
        None => {
            tx.rollback().await.ok();
            return Err(internal_error());
        }
    };
    if st != InvitationStatus::Pending {
        tx.rollback().await.ok();
        return Err(gone_invitation(invitation_inactive_message(st), st));
    }

    let user_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
        .bind(&invitation.invited_email)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| internal_error())?;

    if user_exists {
        tx.rollback().await.ok();
        return Err((
            StatusCode::CONFLICT,
            Json(ApiErrorBody::msg(
                "Please sign in to accept this invitation.",
            )),
        ));
    }

    let password_hash = match hash_password_argon2(password) {
        Ok(h) => h,
        Err(()) => {
            tx.rollback().await.ok();
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
            status,
            timezone
        )
        VALUES ($1, $2, lower(trim($3)), $4, 'company', $5, 'active', $6)
        RETURNING id
        "#,
    )
    .bind(fullname)
    .bind(&invitation.invited_email)
    .bind(username)
    .bind(&password_hash)
    .bind(&invitation.invited_role)
    .bind(timezone)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(id) => id,
        Err(e) => {
            tx.rollback().await.ok();
            if let Some(db) = e.as_database_error() {
                if db.code().as_deref() == Some("23505") {
                    return Err((
                        StatusCode::CONFLICT,
                        Json(ApiErrorBody::msg("This username is already taken.")),
                    ));
                }
            }
            return Err(internal_error());
        }
    };

    accept_invitation_in_tx(&mut tx, &invitation, user_id).await?;

    tx.commit().await.map_err(|_| internal_error())?;

    // SF-24: audit log "Invitation accepted" for this user/company.

    let (jar, body) = auth_route::establish_session_for_user(
        &state.pool,
        state.session_max_age_secs,
        jar,
        user_id,
    )
    .await?;

    Ok((jar, Json(body)))
}

pub async fn accept_invitation_confirm(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<AcceptInvitationConfirmRequest>,
) -> Result<Json<CurrentUserBody>, (StatusCode, Json<ApiErrorBody>)> {
    let user = auth_route::require_authenticated_user(&state.pool, &jar).await?;

    let token_hex = payload.token.trim();
    if token_hex.is_empty() {
        return Err(bad_request("Invitation token is required."));
    }

    let raw = match decode_invitation_token_hex(token_hex) {
        Some(b) => b,
        None => return Err(not_found_invitation()),
    };
    let token_hash = hash_invitation_token(&raw);

    let mut tx = state.pool.begin().await.map_err(|_| internal_error())?;

    let invitation = match lock_pending_invitation(&mut tx, &token_hash).await? {
        Some(inv) => inv,
        None => {
            tx.rollback().await.ok();
            return Err(not_found_invitation());
        }
    };

    if user.email != invitation.invited_email {
        tx.rollback().await.ok();
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiErrorBody::msg(
                "This invitation was sent to a different email address.",
            )),
        ));
    }

    if invitation.expires_at < Utc::now() {
        expire_invitation_if_pending(&mut tx, invitation.id).await?;
        tx.commit().await.map_err(|_| internal_error())?;
        return Err(gone_invitation(
            "This invitation has expired.",
            InvitationStatus::Expired,
        ));
    }

    let st = match InvitationStatus::from_db_value(invitation.status.as_str()) {
        Some(s) => s,
        None => {
            tx.rollback().await.ok();
            return Err(internal_error());
        }
    };
    if st != InvitationStatus::Pending {
        tx.rollback().await.ok();
        return Err(gone_invitation(invitation_inactive_message(st), st));
    }

    let existing_company_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT company_id FROM company_users WHERE user_id = $1",
    )
    .bind(user.id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| internal_error())?
    .flatten();

    if let Some(cid) = existing_company_id {
        if cid != invitation.company_id {
            tx.rollback().await.ok();
            return Err((
                StatusCode::CONFLICT,
                Json(ApiErrorBody::msg(
                    "You already belong to another company. Phase 1 does not support multi-company users.",
                )),
            ));
        }
    }

    sqlx::query(
        r#"
        UPDATE users
        SET account_type = 'company',
            role = $2,
            updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(user.id)
    .bind(&invitation.invited_role)
    .execute(&mut *tx)
    .await
    .map_err(|_| internal_error())?;

    if existing_company_id.is_none() {
        sqlx::query(
            r#"
            INSERT INTO company_users (company_id, user_id)
            VALUES ($1, $2)
            "#,
        )
        .bind(invitation.company_id)
        .bind(user.id)
        .execute(&mut *tx)
        .await
        .map_err(|_| internal_error())?;
    }

    maybe_insert_project_membership(&mut tx, &invitation, user.id).await?;

    let updated_rows = sqlx::query(
        r#"
        UPDATE invitations
        SET status = $2,
            accepted_at = now()
        WHERE id = $1 AND status = $3
        "#,
    )
    .bind(invitation.id)
    .bind(InvitationStatus::Accepted.as_db_value())
    .bind(InvitationStatus::Pending.as_db_value())
    .execute(&mut *tx)
    .await
    .map_err(|_| internal_error())?
    .rows_affected();

    if updated_rows != 1 {
        tx.rollback().await.ok();
        return Err(internal_error());
    }

    tx.commit().await.map_err(|_| internal_error())?;

    // SF-24: audit log "Invitation accepted".

    let body = load_current_user_body(&state.pool, user.id).await?;
    Ok(Json(body))
}

async fn load_current_user_body(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<CurrentUserBody, (StatusCode, Json<ApiErrorBody>)> {
    let row = sqlx::query_as::<_, UserRowForBody>(
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

    let account_type = AccountType::from_db_value(row.account_type.as_str()).ok_or_else(internal_error)?;
    let role = decode_role(row.role.as_deref())?;

    Ok(CurrentUserBody {
        id: row.id,
        fullname: row.fullname,
        email: row.email,
        avatar_url: row.avatar_url,
        account_type,
        role,
        company_id: row.company_id,
    })
}

#[derive(sqlx::FromRow)]
struct UserRowForBody {
    id: Uuid,
    fullname: String,
    email: String,
    avatar_url: Option<String>,
    account_type: String,
    role: Option<String>,
    company_id: Option<Uuid>,
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

async fn lock_pending_invitation(
    tx: &mut Transaction<'_, Postgres>,
    token_hash: &str,
) -> Result<Option<Invitation>, (StatusCode, Json<ApiErrorBody>)> {
    let row = sqlx::query_as::<_, Invitation>(
        r#"
        SELECT
            id,
            invitation_token_hash,
            company_id,
            project_id,
            invited_email,
            invited_role,
            invited_by_user_id,
            status,
            expires_at,
            accepted_at,
            created_at
        FROM invitations
        WHERE invitation_token_hash = $1
        FOR UPDATE
        "#,
    )
    .bind(token_hash)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| internal_error())?;

    Ok(row)
}

async fn expire_invitation_if_pending(
    tx: &mut Transaction<'_, Postgres>,
    invitation_id: Uuid,
) -> Result<(), (StatusCode, Json<ApiErrorBody>)> {
    sqlx::query(
        r#"
        UPDATE invitations
        SET status = $2
        WHERE id = $1 AND status = $3
        "#,
    )
    .bind(invitation_id)
    .bind(InvitationStatus::Expired.as_db_value())
    .bind(InvitationStatus::Pending.as_db_value())
    .execute(&mut **tx)
    .await
    .map_err(|_| internal_error())?;
    Ok(())
}

async fn accept_invitation_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    invitation: &Invitation,
    user_id: Uuid,
) -> Result<(), (StatusCode, Json<ApiErrorBody>)> {
    sqlx::query(
        r#"
        INSERT INTO company_users (company_id, user_id)
        VALUES ($1, $2)
        "#,
    )
    .bind(invitation.company_id)
    .bind(user_id)
    .execute(&mut **tx)
    .await
    .map_err(|_| internal_error())?;

    maybe_insert_project_membership(tx, invitation, user_id).await?;

    let updated_rows = sqlx::query(
        r#"
        UPDATE invitations
        SET status = $2,
            accepted_at = now()
        WHERE id = $1 AND status = $3
        "#,
    )
    .bind(invitation.id)
    .bind(InvitationStatus::Accepted.as_db_value())
    .bind(InvitationStatus::Pending.as_db_value())
    .execute(&mut **tx)
    .await
    .map_err(|_| internal_error())?
    .rows_affected();

    if updated_rows != 1 {
        return Err(internal_error());
    }

    Ok(())
}

async fn maybe_insert_project_membership(
    tx: &mut Transaction<'_, Postgres>,
    invitation: &Invitation,
    user_id: Uuid,
) -> Result<(), (StatusCode, Json<ApiErrorBody>)> {
    let Some(pid) = invitation.project_id else {
        return Ok(());
    };

    let invited_role = match CompanyRole::from_db_value(invitation.invited_role.as_str()) {
        Some(r) => r,
        None => {
            warn!(
                invitation_id = %invitation.id,
                "invitation invited_role invalid; skipping project membership"
            );
            return Ok(());
        }
    };

    let Some(pm_role) = ProjectMembershipRole::from_company_role(invited_role) else {
        return Ok(());
    };

    let ok: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT id
        FROM projects
        WHERE id = $1 AND company_id = $2
        "#,
    )
    .bind(pid)
    .bind(invitation.company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| internal_error())?;

    if ok.is_none() {
        warn!(
            invitation_id = %invitation.id,
            project_id = %pid,
            "invitation project_id missing or not in company; skipping project membership"
        );
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO project_memberships (project_id, user_id, role)
        VALUES ($1, $2, $3)
        ON CONFLICT (project_id, user_id) DO NOTHING
        "#,
    )
    .bind(pid)
    .bind(user_id)
    .bind(pm_role.as_db_value())
    .execute(&mut **tx)
    .await
    .map_err(|_| internal_error())?;

    Ok(())
}

fn not_found_invitation() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiErrorBody::msg(
            "This invitation link is invalid or has expired.",
        )),
    )
}

fn gone_invitation(
    message: &'static str,
    status: InvitationStatus,
) -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::GONE,
        Json(ApiErrorBody::invitation_inactive(message, status)),
    )
}

fn bad_request(message: impl Into<String>) -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ApiErrorBody::msg(message)),
    )
}

fn internal_error() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiErrorBody::msg("Something went wrong. Please try again.")),
    )
}
