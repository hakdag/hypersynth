use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use chrono::{Duration as ChronoDuration, Utc};
use rand_core::{OsRng, RngCore};
use tracing::warn;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth_route;
use crate::authorization;
use crate::email::InvitationEmail;
use crate::invitation_token_service::hash_invitation_token;
use crate::tenant_scope_service::TenantScopeService;
use crate::tx_extractor::missing_tx_error;
use crate::types::{
    ApiErrorBody, CompanyRole, CreateInvitationRequest, Invitation, InvitationResponse,
    InvitationStatus, TenantScope, Tx,
};
use crate::user_registration::email_contains_at_and_dot;

fn company_role_invite_label(role: CompanyRole) -> &'static str {
    match role {
        CompanyRole::CompanyAdmin => "Company Admin",
        CompanyRole::ProjectManager => "Project Manager",
        CompanyRole::Contributor => "Contributor",
        CompanyRole::Viewer => "Viewer",
    }
}

fn invited_role_allowed_for_inviter(inviter: CompanyRole, invited: CompanyRole) -> bool {
    match inviter {
        CompanyRole::CompanyAdmin => true,
        CompanyRole::ProjectManager => {
            matches!(invited, CompanyRole::Contributor | CompanyRole::Viewer)
        }
        CompanyRole::Contributor | CompanyRole::Viewer => false,
    }
}

fn invitation_to_response(
    inv: Invitation,
) -> Result<InvitationResponse, (StatusCode, Json<ApiErrorBody>)> {
    let invited_role =
        CompanyRole::from_db_value(inv.invited_role.as_str()).ok_or_else(internal_error)?;
    let status = InvitationStatus::from_db_value(inv.status.as_str()).ok_or_else(internal_error)?;
    Ok(InvitationResponse {
        id: inv.id,
        company_id: inv.company_id,
        project_id: inv.project_id,
        invited_email: inv.invited_email,
        invited_role,
        invited_by_user_id: inv.invited_by_user_id,
        status,
        expires_at: inv.expires_at,
        accepted_at: inv.accepted_at,
        created_at: inv.created_at,
    })
}

pub async fn create_invitation(
    State(state): State<AppState>,
    tx: Tx,
    jar: CookieJar,
    Json(payload): Json<CreateInvitationRequest>,
) -> Result<(StatusCode, Json<InvitationResponse>), (StatusCode, Json<ApiErrorBody>)> {
    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;

    let user = auth_route::require_authenticated_user(conn, &jar).await?;
    authorization::require_company_role(&user, authorization::INVITE_USERS).await?;

    let scope = TenantScopeService::from_session(&user)?;
    let TenantScope::Company {
        company_id,
        role: inviter_role,
        ..
    } = scope
    else {
        return Err(authorization::forbidden(
            "You do not have permission to perform this action.",
        ));
    };

    if !invited_role_allowed_for_inviter(inviter_role, payload.invited_role) {
        return Err(authorization::forbidden(
            "You do not have permission to assign this role.",
        ));
    }

    let email_raw = payload.invited_email.trim();
    if email_raw.is_empty() {
        return Err(bad_request("Email is required."));
    }
    if !email_contains_at_and_dot(email_raw) {
        return Err(bad_request("Enter a valid email address."));
    }
    let invited_email = email_raw.to_lowercase();

    if let Some(pid) = payload.project_id {
        let ok: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT id
            FROM projects
            WHERE id = $1 AND company_id = $2
            "#,
        )
        .bind(pid)
        .bind(company_id)
        .fetch_optional(&mut **conn)
        .await
        .map_err(|_| internal_error())?;
        if ok.is_none() {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ApiErrorBody {
                    message: "Project not found.".into(),
                    ..Default::default()
                }),
            ));
        }
    }

    let company_name: String = sqlx::query_scalar("SELECT name FROM companies WHERE id = $1")
        .bind(company_id)
        .fetch_optional(&mut **conn)
        .await
        .map_err(|_| internal_error())?
        .ok_or_else(internal_error)?;

    let project_name: Option<String> = if let Some(pid) = payload.project_id {
        sqlx::query_scalar("SELECT name FROM projects WHERE id = $1 AND company_id = $2")
            .bind(pid)
            .bind(company_id)
            .fetch_optional(&mut **conn)
            .await
            .map_err(|_| internal_error())?
    } else {
        None
    };

    let mut raw = [0u8; 32];
    OsRng.fill_bytes(&mut raw);
    let token_hex = hex::encode(raw);
    let token_hash = hash_invitation_token(&raw);

    let expires_at = Utc::now() + ChronoDuration::hours(state.invitation_config.expires_in_hours);
    let accept_url = format!(
        "{}/invitations/accept?id={token_hex}",
        state.invitation_config.app_base_url
    );

    let insert_result = sqlx::query_as::<_, Invitation>(
        r#"
        INSERT INTO invitations (
            invitation_token_hash,
            company_id,
            project_id,
            invited_email,
            invited_role,
            invited_by_user_id,
            status,
            expires_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING
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
        "#,
    )
    .bind(&token_hash)
    .bind(company_id)
    .bind(payload.project_id)
    .bind(&invited_email)
    .bind(payload.invited_role.as_db_value())
    .bind(user.id)
    .bind(InvitationStatus::Pending.as_db_value())
    .bind(expires_at)
    .fetch_one(&mut **conn)
    .await;

    let invitation = match insert_result {
        Ok(row) => row,
        Err(e) => {
            if let Some(db) = e.as_database_error() {
                if db.code().as_deref() == Some("23505") {
                    return Err((
                        StatusCode::CONFLICT,
                        Json(ApiErrorBody {
                            message: "An invitation is already pending for this email address."
                                .into(),
                            ..Default::default()
                        }),
                    ));
                }
            }
            return Err(internal_error());
        }
    };

    let email_payload = InvitationEmail {
        company_name,
        inviter_name: user.fullname,
        invited_role_label: company_role_invite_label(payload.invited_role).to_string(),
        project_name,
        accept_url,
        expires_at: invitation.expires_at,
        message: payload.message,
    };

    if let Err(err) = state
        .email_sender
        .send_invitation(invited_email.as_str(), email_payload)
        .await
    {
        warn!(%err, invitation_id = %invitation.id, "invitation email send failed");
        let _ = sqlx::query(
            r#"
            UPDATE invitations
            SET status = $3
            WHERE id = $1 AND company_id = $2 AND status = $4
            "#,
        )
        .bind(invitation.id)
        .bind(company_id)
        .bind(InvitationStatus::Cancelled.as_db_value())
        .bind(InvitationStatus::Pending.as_db_value())
        .execute(&mut **conn)
        .await;

        return Err((
            StatusCode::BAD_GATEWAY,
            Json(ApiErrorBody {
                message: "Invitation was created but the invitation email could not be sent. Please try again or contact support.".into(),
                ..Default::default()
            }),
        ));
    }

    let body = invitation_to_response(invitation)?;
    Ok((StatusCode::CREATED, Json(body)))
}

pub async fn list_invitations(
    tx: Tx,
    jar: CookieJar,
) -> Result<Json<Vec<InvitationResponse>>, (StatusCode, Json<ApiErrorBody>)> {
    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;

    let user = auth_route::require_authenticated_user(conn, &jar).await?;
    authorization::require_company_role(&user, authorization::INVITE_USERS).await?;

    let scope = TenantScopeService::from_session(&user)?;
    let TenantScope::Company { company_id, .. } = scope else {
        return Err(authorization::forbidden(
            "You do not have permission to perform this action.",
        ));
    };

    let _ = sqlx::query(
        r#"
        UPDATE invitations
        SET status = $2
        WHERE company_id = $1
          AND status = $3
          AND expires_at < now()
        "#,
    )
    .bind(company_id)
    .bind(InvitationStatus::Expired.as_db_value())
    .bind(InvitationStatus::Pending.as_db_value())
    .execute(&mut **conn)
    .await
    .map_err(|_| internal_error())?;

    let rows = sqlx::query_as::<_, Invitation>(
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
        WHERE company_id = $1 AND invited_by_user_id = $2
        ORDER BY created_at DESC
        "#,
    )
    .bind(company_id)
    .bind(user.id)
    .fetch_all(&mut **conn)
    .await
    .map_err(|_| internal_error())?;

    let mut out = Vec::with_capacity(rows.len());
    for inv in rows {
        out.push(invitation_to_response(inv)?);
    }

    Ok(Json(out))
}

pub async fn cancel_invitation(
    tx: Tx,
    jar: CookieJar,
    Path(invitation_id): Path<Uuid>,
) -> Result<Json<InvitationResponse>, (StatusCode, Json<ApiErrorBody>)> {
    let mut guard = tx.0.lock().await;
    let conn = guard.as_mut().ok_or_else(missing_tx_error)?;

    let user = auth_route::require_authenticated_user(conn, &jar).await?;
    authorization::require_company_role(&user, authorization::INVITE_USERS).await?;

    let scope = TenantScopeService::from_session(&user)?;
    let TenantScope::Company { company_id, .. } = scope else {
        return Err(authorization::forbidden(
            "You do not have permission to perform this action.",
        ));
    };

    let updated = sqlx::query_as::<_, Invitation>(
        r#"
        UPDATE invitations
        SET status = $4
        WHERE id = $1
          AND company_id = $2
          AND invited_by_user_id = $3
          AND status = $5
        RETURNING
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
        "#,
    )
    .bind(invitation_id)
    .bind(company_id)
    .bind(user.id)
    .bind(InvitationStatus::Cancelled.as_db_value())
    .bind(InvitationStatus::Pending.as_db_value())
    .fetch_optional(&mut **conn)
    .await
    .map_err(|_| internal_error())?
    .ok_or_else(not_found)?;

    Ok(Json(invitation_to_response(updated)?))
}

fn not_found() -> (StatusCode, Json<ApiErrorBody>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiErrorBody {
            message: "Invitation not found.".into(),
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
