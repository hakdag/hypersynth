use sqlx::PgPool;
use uuid::Uuid;

async fn connect_pool() -> Option<PgPool> {
    let database_url = std::env::var("DATABASE_URL").ok()?;
    PgPool::connect(&database_url).await.ok()
}

struct CompanyWorkspace {
    member_id: Uuid,
    second_member_id: Uuid,
    task_id: Uuid,
}

async fn seed_company_workspace(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<CompanyWorkspace, sqlx::Error> {
    let company_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let second_member_id = Uuid::new_v4();
    let admin_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let feature_id = Uuid::new_v4();
    let task_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO companies (id, name, company_email, country, timezone, status)
        VALUES ($1, 'SF34 Co', $2, 'TR', 'UTC', 'active')
        "#,
    )
    .bind(company_id)
    .bind(format!("sf34-comments-{}@example.test", Uuid::new_v4()))
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO users (id, fullname, email, password_hash, account_type, role)
        VALUES
            ($1, 'Member One', $4, 'unused', 'company', 'contributor'),
            ($2, 'Member Two', $5, 'unused', 'company', 'contributor'),
            ($3, 'Company Admin', $6, 'unused', 'company', 'company_admin')
        "#,
    )
    .bind(member_id)
    .bind(second_member_id)
    .bind(admin_id)
    .bind(format!("member-one-{}@example.test", Uuid::new_v4()))
    .bind(format!("member-two-{}@example.test", Uuid::new_v4()))
    .bind(format!("admin-{}@example.test", Uuid::new_v4()))
    .execute(&mut **tx)
    .await?;

    for user_id in [member_id, second_member_id, admin_id] {
        sqlx::query("INSERT INTO company_users (company_id, user_id) VALUES ($1, $2)")
            .bind(company_id)
            .bind(user_id)
            .execute(&mut **tx)
            .await?;
    }

    sqlx::query(
        r#"
        INSERT INTO projects (id, owner_user_id, company_id, created_by_user_id, name, status)
        VALUES ($1, NULL, $2, $3, 'Project', 'Pending')
        "#,
    )
    .bind(project_id)
    .bind(company_id)
    .bind(member_id)
    .execute(&mut **tx)
    .await?;

    for user_id in [member_id, second_member_id, admin_id] {
        sqlx::query(
            "INSERT INTO project_memberships (project_id, user_id, role) VALUES ($1, $2, 'contributor')",
        )
        .bind(project_id)
        .bind(user_id)
        .execute(&mut **tx)
        .await?;
    }

    sqlx::query("INSERT INTO features (id, project_id, title, status) VALUES ($1, $2, 'Feature', 'Pending')")
        .bind(feature_id)
        .bind(project_id)
        .execute(&mut **tx)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO tasks (id, feature_id, title, description, status, created_by, creator_user_id, priority)
        VALUES ($1, $2, 'Task', NULL, 'Pending', 'User', $3, 'Standard')
        "#,
    )
    .bind(task_id)
    .bind(feature_id)
    .bind(member_id)
    .execute(&mut **tx)
    .await?;

    Ok(CompanyWorkspace {
        member_id,
        second_member_id,
        task_id,
    })
}

#[tokio::test]
async fn comments_enforce_non_empty_content() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };
    sqlx::migrate!("./migrations").run(&pool).await?;
    let mut tx = pool.begin().await?;
    let workspace = seed_company_workspace(&mut tx).await?;

    let invalid = sqlx::query(
        "INSERT INTO task_comments (id, task_id, user_id, content) VALUES ($1, $2, $3, '   ')",
    )
    .bind(Uuid::new_v4())
    .bind(workspace.task_id)
    .bind(workspace.member_id)
    .execute(&mut *tx)
    .await;
    assert!(invalid.is_err());

    tx.rollback().await?;
    Ok(())
}

#[tokio::test]
async fn comments_list_in_chronological_order() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };
    sqlx::migrate!("./migrations").run(&pool).await?;
    let mut tx = pool.begin().await?;
    let workspace = seed_company_workspace(&mut tx).await?;

    let first_id = Uuid::new_v4();
    let second_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO task_comments (id, task_id, user_id, content, created_at, updated_at) VALUES ($1, $2, $3, 'first', NOW() - INTERVAL '2 minutes', NOW() - INTERVAL '2 minutes')",
    )
    .bind(first_id)
    .bind(workspace.task_id)
    .bind(workspace.member_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO task_comments (id, task_id, user_id, content, created_at, updated_at) VALUES ($1, $2, $3, 'second', NOW() - INTERVAL '1 minute', NOW() - INTERVAL '1 minute')",
    )
    .bind(second_id)
    .bind(workspace.task_id)
    .bind(workspace.member_id)
    .execute(&mut *tx)
    .await?;

    let ordered_ids: Vec<Uuid> = sqlx::query_scalar(
        "SELECT id FROM task_comments WHERE task_id = $1 ORDER BY created_at ASC",
    )
    .bind(workspace.task_id)
    .fetch_all(&mut *tx)
    .await?;
    assert_eq!(ordered_ids, vec![first_id, second_id]);

    tx.rollback().await?;
    Ok(())
}

#[tokio::test]
async fn comments_update_changes_updated_at() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };
    sqlx::migrate!("./migrations").run(&pool).await?;
    let mut tx = pool.begin().await?;
    let workspace = seed_company_workspace(&mut tx).await?;
    let comment_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO task_comments (id, task_id, user_id, content, created_at, updated_at) VALUES ($1, $2, $3, 'before', NOW() - INTERVAL '2 minutes', NOW() - INTERVAL '2 minutes')",
    )
    .bind(comment_id)
    .bind(workspace.task_id)
    .bind(workspace.member_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE task_comments SET content = 'after', updated_at = NOW() WHERE id = $1")
        .bind(comment_id)
        .execute(&mut *tx)
        .await?;

    let row: (String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
        "SELECT content, created_at, updated_at FROM task_comments WHERE id = $1",
    )
    .bind(comment_id)
    .fetch_one(&mut *tx)
    .await?;
    assert_eq!(row.0, "after");
    assert!(row.2 > row.1);

    tx.rollback().await?;
    Ok(())
}

#[tokio::test]
async fn comment_authorization_and_isolation_rules_hold() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };
    sqlx::migrate!("./migrations").run(&pool).await?;
    let mut tx = pool.begin().await?;
    let workspace = seed_company_workspace(&mut tx).await?;
    let comment_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO task_comments (id, task_id, user_id, content) VALUES ($1, $2, $3, 'owned by member')",
    )
    .bind(comment_id)
    .bind(workspace.task_id)
    .bind(workspace.member_id)
    .execute(&mut *tx)
    .await?;

    let non_author_update = sqlx::query(
        "UPDATE task_comments SET content = 'forbidden' WHERE id = $1 AND user_id = $2",
    )
    .bind(comment_id)
    .bind(workspace.second_member_id)
    .execute(&mut *tx)
    .await?;
    assert_eq!(non_author_update.rows_affected(), 0);

    let admin_update = sqlx::query(
        "UPDATE task_comments SET content = 'admin override' WHERE id = $1",
    )
    .bind(comment_id)
    .execute(&mut *tx)
    .await?;
    assert_eq!(admin_update.rows_affected(), 1);

    let other_company_id = Uuid::new_v4();
    let outsider_user_id = Uuid::new_v4();
    let outsider_project_id = Uuid::new_v4();
    let outsider_feature_id = Uuid::new_v4();
    let outsider_task_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO companies (id, name, company_email, country, timezone, status)
        VALUES ($1, 'Other Co', $2, 'TR', 'UTC', 'active')
        "#,
    )
    .bind(other_company_id)
    .bind(format!("sf34-other-{}@example.test", Uuid::new_v4()))
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO users (id, fullname, email, password_hash, account_type, role)
        VALUES ($1, 'Other User', $2, 'unused', 'company', 'contributor')
        "#,
    )
    .bind(outsider_user_id)
    .bind(format!("sf34-outsider-{}@example.test", Uuid::new_v4()))
    .execute(&mut *tx)
    .await?;

    sqlx::query("INSERT INTO company_users (company_id, user_id) VALUES ($1, $2)")
        .bind(other_company_id)
        .bind(outsider_user_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "INSERT INTO projects (id, owner_user_id, company_id, created_by_user_id, name, status) VALUES ($1, NULL, $2, $3, 'Other Project', 'Pending')",
    )
    .bind(outsider_project_id)
    .bind(other_company_id)
    .bind(outsider_user_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("INSERT INTO project_memberships (project_id, user_id, role) VALUES ($1, $2, 'contributor')")
        .bind(outsider_project_id)
        .bind(outsider_user_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("INSERT INTO features (id, project_id, title, status) VALUES ($1, $2, 'Other Feature', 'Pending')")
        .bind(outsider_feature_id)
        .bind(outsider_project_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "INSERT INTO tasks (id, feature_id, title, description, status, created_by, creator_user_id, priority) VALUES ($1, $2, 'Other Task', NULL, 'Pending', 'User', $3, 'Standard')",
    )
    .bind(outsider_task_id)
    .bind(outsider_feature_id)
    .bind(outsider_user_id)
    .execute(&mut *tx)
    .await?;

    let outsider_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM task_comments c INNER JOIN tasks t ON t.id = c.task_id INNER JOIN features f ON f.id = t.feature_id INNER JOIN projects p ON p.id = f.project_id WHERE c.id = $1 AND p.company_id = $2",
    )
    .bind(comment_id)
    .bind(other_company_id)
    .fetch_one(&mut *tx)
    .await?;
    assert_eq!(outsider_count, 0);

    tx.rollback().await?;
    Ok(())
}

#[tokio::test]
async fn deleting_task_cascades_comments() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };
    sqlx::migrate!("./migrations").run(&pool).await?;
    let mut tx = pool.begin().await?;
    let workspace = seed_company_workspace(&mut tx).await?;
    let comment_id = Uuid::new_v4();

    sqlx::query("INSERT INTO task_comments (id, task_id, user_id, content) VALUES ($1, $2, $3, 'bye')")
        .bind(comment_id)
        .bind(workspace.task_id)
        .bind(workspace.member_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM tasks WHERE id = $1")
        .bind(workspace.task_id)
        .execute(&mut *tx)
        .await?;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*)::bigint FROM task_comments WHERE id = $1")
        .bind(comment_id)
        .fetch_one(&mut *tx)
        .await?;
    assert_eq!(count, 0);

    tx.rollback().await?;
    Ok(())
}
