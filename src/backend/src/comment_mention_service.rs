use std::collections::{HashMap, HashSet};

use axum::http::StatusCode;
use axum::Json;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::types::{ApiErrorBody, CommentMentionSummary, TenantScope};

#[derive(Debug, Clone)]
pub struct ResolvedMention {
    pub user_id: Uuid,
    pub username: String,
    pub fullname: String,
}

pub fn parse_mention_usernames(content: &str) -> Vec<String> {
    let chars: Vec<char> = content.chars().collect();
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let mut idx = 0usize;

    while idx < chars.len() {
        if chars[idx] != '@' {
            idx += 1;
            continue;
        }

        let mut end = idx + 1;
        while end < chars.len() && is_username_char(chars[end]) {
            end += 1;
        }

        let len = end.saturating_sub(idx + 1);
        if (3..=64).contains(&len) {
            let username: String = chars[idx + 1..end].iter().collect();
            let normalized = username.to_ascii_lowercase();
            if seen.insert(normalized.clone()) {
                out.push(normalized);
            }
        }

        idx = end;
    }

    out
}

pub async fn resolve_mentions(
    conn: &mut PgConnection,
    scope: TenantScope,
    project_id: Uuid,
    usernames_lower: &[String],
) -> Result<Vec<ResolvedMention>, (StatusCode, Json<ApiErrorBody>)> {
    let deduped = dedupe_lower_usernames(usernames_lower);
    if deduped.is_empty() {
        return Ok(Vec::new());
    }

    let rows = match scope {
        TenantScope::Personal { user_id } => {
            sqlx::query_as::<_, (Uuid, String, String)>(
                r#"
                SELECT u.id, u.username, u.fullname
                FROM users u
                INNER JOIN projects p ON p.id = $2
                WHERE p.owner_user_id = $1
                  AND p.company_id IS NULL
                  AND u.id = $1
                  AND u.username IS NOT NULL
                  AND lower(u.username) = ANY($3)
                "#,
            )
            .bind(user_id)
            .bind(project_id)
            .bind(&deduped)
            .fetch_all(&mut *conn)
            .await
            .map_err(|_| internal_error())?
        }
        TenantScope::Company { company_id, .. } => {
            sqlx::query_as::<_, (Uuid, String, String)>(
                r#"
                SELECT u.id, u.username, u.fullname
                FROM users u
                INNER JOIN company_users cu ON cu.user_id = u.id
                INNER JOIN project_memberships pm ON pm.user_id = u.id
                WHERE cu.company_id = $1
                  AND pm.project_id = $2
                  AND u.username IS NOT NULL
                  AND lower(u.username) = ANY($3)
                "#,
            )
            .bind(company_id)
            .bind(project_id)
            .bind(&deduped)
            .fetch_all(&mut *conn)
            .await
            .map_err(|_| internal_error())?
        }
    };

    let mut by_username = HashMap::new();
    for (user_id, username, fullname) in rows {
        by_username.insert(
            username.to_ascii_lowercase(),
            ResolvedMention {
                user_id,
                username,
                fullname,
            },
        );
    }

    let mut out = Vec::new();
    for username in deduped {
        if let Some(resolved) = by_username.remove(&username) {
            out.push(resolved);
        }
    }

    Ok(out)
}

pub async fn sync_comment_mentions(
    conn: &mut PgConnection,
    comment_id: Uuid,
    user_ids: &[Uuid],
) -> Result<(), (StatusCode, Json<ApiErrorBody>)> {
    let deduped = dedupe_user_ids(user_ids);
    sqlx::query("DELETE FROM task_comment_mentions WHERE comment_id = $1")
        .bind(comment_id)
        .execute(&mut *conn)
        .await
        .map_err(|_| internal_error())?;

    for user_id in deduped {
        sqlx::query(
            r#"
            INSERT INTO task_comment_mentions (comment_id, user_id)
            VALUES ($1, $2)
            ON CONFLICT (comment_id, user_id) DO NOTHING
            "#,
        )
        .bind(comment_id)
        .bind(user_id)
        .execute(&mut *conn)
        .await
        .map_err(|_| internal_error())?;
    }

    Ok(())
}

pub async fn fetch_mentions_for_comments(
    conn: &mut PgConnection,
    comment_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<CommentMentionSummary>>, (StatusCode, Json<ApiErrorBody>)> {
    let deduped = dedupe_user_ids(comment_ids);
    if deduped.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query_as::<_, (Uuid, Uuid, String, String)>(
        r#"
        SELECT m.comment_id, u.id, u.username, u.fullname
        FROM task_comment_mentions m
        INNER JOIN users u ON u.id = m.user_id
        WHERE m.comment_id = ANY($1)
        ORDER BY m.comment_id ASC, lower(u.username) ASC
        "#,
    )
    .bind(&deduped)
    .fetch_all(&mut *conn)
    .await
    .map_err(|_| internal_error())?;

    let mut out: HashMap<Uuid, Vec<CommentMentionSummary>> = HashMap::new();
    for (comment_id, user_id, username, fullname) in rows {
        out.entry(comment_id).or_default().push(CommentMentionSummary {
            user_id,
            username,
            fullname,
        });
    }
    Ok(out)
}

fn dedupe_user_ids(ids: &[Uuid]) -> Vec<Uuid> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for id in ids {
        if seen.insert(*id) {
            out.push(*id);
        }
    }
    out
}

fn dedupe_lower_usernames(usernames: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for username in usernames {
        let normalized = username.to_ascii_lowercase();
        if seen.insert(normalized.clone()) {
            out.push(normalized);
        }
    }
    out
}

fn is_username_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' || ch == '-'
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
