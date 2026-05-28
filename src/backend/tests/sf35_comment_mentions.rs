use sqlx::PgPool;
use uuid::Uuid;

async fn connect_pool() -> Option<PgPool> {
    let database_url = std::env::var("DATABASE_URL").ok()?;
    PgPool::connect(&database_url).await.ok()
}

struct CompanyWorkspace {
    company_id: Uuid,
    project_id: Uuid,
    task_id: Uuid,
    member_one_id: Uuid,
    member_two_id: Uuid,
}

async fn seed_company_workspace(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<CompanyWorkspace, sqlx::Error> {
    let company_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let feature_id = Uuid::new_v4();
    let task_id = Uuid::new_v4();
    let member_one_id = Uuid::new_v4();
    let member_two_id = Uuid::new_v4();
    let outsider_company_user_id = Uuid::new_v4();
    let other_company_id = Uuid::new_v4();
    let other_company_member_two_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO companies (id, name, company_email, country, timezone, status)
        VALUES ($1, 'Mention Co', $2, 'TR', 'UTC', 'active')
        "#,
    )
    .bind(company_id)
    .bind(format!("mention-company-{}@example.test", Uuid::new_v4()))
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO companies (id, name, company_email, country, timezone, status)
        VALUES ($1, 'Other Co', $2, 'TR', 'UTC', 'active')
        "#,
    )
    .bind(other_company_id)
    .bind(format!("mention-other-company-{}@example.test", Uuid::new_v4()))
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO users (id, fullname, email, username, password_hash, account_type, role)
        VALUES
            ($1, 'Member One', $4, 'member_one', 'unused', 'company', 'contributor'),
            ($2, 'Member Two', $5, 'member_two', 'unused', 'company', 'contributor'),
            ($3, 'Outsider Same Company', $6, 'outsider_same_company', 'unused', 'company', 'contributor'),
            ($7, 'Other Company Member Two', $8, 'member_two', 'unused', 'company', 'contributor')
        "#,
    )
    .bind(member_one_id)
    .bind(member_two_id)
    .bind(outsider_company_user_id)
    .bind(format!("member-one-{}@example.test", Uuid::new_v4()))
    .bind(format!("member-two-{}@example.test", Uuid::new_v4()))
    .bind(format!("outsider-same-co-{}@example.test", Uuid::new_v4()))
    .bind(other_company_member_two_id)
    .bind(format!("other-co-member-two-{}@example.test", Uuid::new_v4()))
    .execute(&mut **tx)
    .await?;

    for user_id in [member_one_id, member_two_id, outsider_company_user_id] {
        sqlx::query("INSERT INTO company_users (company_id, user_id) VALUES ($1, $2)")
            .bind(company_id)
            .bind(user_id)
            .execute(&mut **tx)
            .await?;
    }
    sqlx::query("INSERT INTO company_users (company_id, user_id) VALUES ($1, $2)")
        .bind(other_company_id)
        .bind(other_company_member_two_id)
        .execute(&mut **tx)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO projects (id, owner_user_id, company_id, created_by_user_id, name, status)
        VALUES ($1, NULL, $2, $3, 'Mention Project', 'Pending')
        "#,
    )
    .bind(project_id)
    .bind(company_id)
    .bind(member_one_id)
    .execute(&mut **tx)
    .await?;

    for user_id in [member_one_id, member_two_id] {
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
    .bind(member_one_id)
    .execute(&mut **tx)
    .await?;

    Ok(CompanyWorkspace {
        company_id,
        project_id,
        task_id,
        member_one_id,
        member_two_id,
    })
}

#[tokio::test]
async fn mention_resolution_keeps_only_scoped_project_members() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };
    sqlx::migrate!("./migrations").run(&pool).await?;
    let mut tx = pool.begin().await?;
    let workspace = seed_company_workspace(&mut tx).await?;

    let resolved: Vec<Uuid> = sqlx::query_scalar(
        r#"
        SELECT u.id
        FROM users u
        INNER JOIN company_users cu ON cu.user_id = u.id
        INNER JOIN project_memberships pm ON pm.user_id = u.id
        WHERE cu.company_id = $1
          AND pm.project_id = $2
          AND lower(u.username) = ANY($3)
        ORDER BY lower(u.username) ASC
        "#,
    )
    .bind(workspace.company_id)
    .bind(workspace.project_id)
    .bind(vec![
        "member_two".to_string(),
        "unknown".to_string(),
        "outsider_same_company".to_string(),
    ])
    .fetch_all(&mut *tx)
    .await?;

    assert_eq!(resolved, vec![workspace.member_two_id]);

    tx.rollback().await?;
    Ok(())
}

#[tokio::test]
async fn mention_sync_dedupes_and_replaces_on_edit() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };
    sqlx::migrate!("./migrations").run(&pool).await?;
    let mut tx = pool.begin().await?;
    let workspace = seed_company_workspace(&mut tx).await?;
    let comment_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO task_comments (id, task_id, user_id, content) VALUES ($1, $2, $3, 'ping @member_two @member_two')",
    )
    .bind(comment_id)
    .bind(workspace.task_id)
    .bind(workspace.member_one_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO task_comment_mentions (comment_id, user_id)
        VALUES ($1, $2), ($1, $2)
        ON CONFLICT (comment_id, user_id) DO NOTHING
        "#,
    )
    .bind(comment_id)
    .bind(workspace.member_two_id)
    .execute(&mut *tx)
    .await?;

    let first_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM task_comment_mentions WHERE comment_id = $1",
    )
    .bind(comment_id)
    .fetch_one(&mut *tx)
    .await?;
    assert_eq!(first_count, 1);

    sqlx::query("DELETE FROM task_comment_mentions WHERE comment_id = $1")
        .bind(comment_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO task_comment_mentions (comment_id, user_id) VALUES ($1, $2)",
    )
    .bind(comment_id)
    .bind(workspace.member_one_id)
    .execute(&mut *tx)
    .await?;

    let after_edit: Vec<Uuid> = sqlx::query_scalar(
        "SELECT user_id FROM task_comment_mentions WHERE comment_id = $1",
    )
    .bind(comment_id)
    .fetch_all(&mut *tx)
    .await?;
    assert_eq!(after_edit, vec![workspace.member_one_id]);

    tx.rollback().await?;
    Ok(())
}

#[tokio::test]
async fn mention_rows_cascade_on_comment_delete() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };
    sqlx::migrate!("./migrations").run(&pool).await?;
    let mut tx = pool.begin().await?;
    let workspace = seed_company_workspace(&mut tx).await?;
    let comment_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO task_comments (id, task_id, user_id, content) VALUES ($1, $2, $3, 'hello')",
    )
    .bind(comment_id)
    .bind(workspace.task_id)
    .bind(workspace.member_one_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO task_comment_mentions (comment_id, user_id) VALUES ($1, $2)",
    )
    .bind(comment_id)
    .bind(workspace.member_two_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM task_comments WHERE id = $1")
        .bind(comment_id)
        .execute(&mut *tx)
        .await?;

    let remaining: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM task_comment_mentions WHERE comment_id = $1",
    )
    .bind(comment_id)
    .fetch_one(&mut *tx)
    .await?;
    assert_eq!(remaining, 0);

    tx.rollback().await?;
    Ok(())
}

#[tokio::test]
async fn personal_scope_mentions_only_owner_and_drops_unknown() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };
    sqlx::migrate!("./migrations").run(&pool).await?;
    let mut tx = pool.begin().await?;
    let owner_id = Uuid::new_v4();
    let other_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO users (id, fullname, email, username, password_hash, account_type)
        VALUES
            ($1, 'Owner', $3, 'personal_owner', 'unused', 'individual'),
            ($2, 'Other', $4, 'personal_other', 'unused', 'individual')
        "#,
    )
    .bind(owner_id)
    .bind(other_id)
    .bind(format!("owner-{}@example.test", Uuid::new_v4()))
    .bind(format!("other-{}@example.test", Uuid::new_v4()))
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO projects (id, owner_user_id, company_id, created_by_user_id, name, status)
        VALUES ($1, $2, NULL, $2, 'Personal Project', 'Pending')
        "#,
    )
    .bind(project_id)
    .bind(owner_id)
    .execute(&mut *tx)
    .await?;

    let resolved: Vec<Uuid> = sqlx::query_scalar(
        r#"
        SELECT u.id
        FROM users u
        INNER JOIN projects p ON p.id = $1
        WHERE p.owner_user_id = $2
          AND p.company_id IS NULL
          AND u.id = $2
          AND lower(u.username) = ANY($3)
        "#,
    )
    .bind(project_id)
    .bind(owner_id)
    .bind(vec![
        "personal_owner".to_string(),
        "personal_other".to_string(),
        "unknown".to_string(),
    ])
    .fetch_all(&mut *tx)
    .await?;

    assert_eq!(resolved, vec![owner_id]);

    tx.rollback().await?;
    Ok(())
}
