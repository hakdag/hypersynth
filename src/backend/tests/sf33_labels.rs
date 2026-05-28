use sqlx::PgPool;
use uuid::Uuid;

async fn connect_pool() -> Option<PgPool> {
    let database_url = std::env::var("DATABASE_URL").ok()?;
    PgPool::connect(&database_url).await.ok()
}

async fn seed_company_workspace(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(Uuid, Uuid, Uuid, Uuid), sqlx::Error> {
    let company_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let feature_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO companies (id, name, company_email, country, timezone, status)
        VALUES ($1, 'SF33 Co', $2, 'TR', 'UTC', 'active')
        "#,
    )
    .bind(company_id)
    .bind(format!("sf33-labels-{}@example.test", Uuid::new_v4()))
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO users (id, fullname, email, password_hash, account_type, role)
        VALUES ($1, 'Member', $2, 'unused', 'company', 'contributor')
        "#,
    )
    .bind(user_id)
    .bind(format!("member-{}@example.test", Uuid::new_v4()))
    .execute(&mut **tx)
    .await?;

    sqlx::query("INSERT INTO company_users (company_id, user_id) VALUES ($1, $2)")
        .bind(company_id)
        .bind(user_id)
        .execute(&mut **tx)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO projects (id, owner_user_id, company_id, created_by_user_id, name, status)
        VALUES ($1, NULL, $2, $3, 'Project', 'Pending')
        "#,
    )
    .bind(project_id)
    .bind(company_id)
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    sqlx::query("INSERT INTO project_memberships (project_id, user_id, role) VALUES ($1, $2, 'contributor')")
        .bind(project_id)
        .bind(user_id)
        .execute(&mut **tx)
        .await?;

    sqlx::query("INSERT INTO features (id, project_id, title, status) VALUES ($1, $2, 'Feature', 'Pending')")
        .bind(feature_id)
        .bind(project_id)
        .execute(&mut **tx)
        .await?;

    Ok((company_id, user_id, project_id, feature_id))
}

#[tokio::test]
async fn labels_enforce_scope_xor_color_and_unique_name() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };
    sqlx::migrate!("./migrations").run(&pool).await?;

    let mut tx = pool.begin().await?;
    let (company_id, user_id, _project_id, _feature_id) = seed_company_workspace(&mut tx).await?;

    let label_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO labels (id, name, color, company_id, user_id)
        VALUES ($1, 'Backend', '#A1B2C3', $2, NULL)
        "#,
    )
    .bind(label_id)
    .bind(company_id)
    .execute(&mut *tx)
    .await?;

    let duplicate_name = sqlx::query(
        r#"
        INSERT INTO labels (id, name, color, company_id, user_id)
        VALUES ($1, 'backend', '#C3B2A1', $2, NULL)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(company_id)
    .execute(&mut *tx)
    .await;
    assert!(duplicate_name.is_err());

    let invalid_color = sqlx::query(
        r#"
        INSERT INTO labels (id, name, color, company_id, user_id)
        VALUES ($1, 'Ops', 'green', $2, NULL)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(company_id)
    .execute(&mut *tx)
    .await;
    assert!(invalid_color.is_err());

    let invalid_scope = sqlx::query(
        r#"
        INSERT INTO labels (id, name, color, company_id, user_id)
        VALUES ($1, 'Personal+Company', '#112233', $2, $3)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(company_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await;
    assert!(invalid_scope.is_err());

    tx.rollback().await?;
    Ok(())
}

#[tokio::test]
async fn task_labels_support_m2m_and_cascade_on_label_delete() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };
    sqlx::migrate!("./migrations").run(&pool).await?;

    let mut tx = pool.begin().await?;
    let (company_id, _user_id, _project_id, feature_id) = seed_company_workspace(&mut tx).await?;

    let label_id = Uuid::new_v4();
    let task_a = Uuid::new_v4();
    let task_b = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO labels (id, name, color, company_id, user_id)
        VALUES ($1, 'Platform', '#445566', $2, NULL)
        "#,
    )
    .bind(label_id)
    .bind(company_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO tasks (id, feature_id, title, description, status, created_by)
        VALUES ($1, $2, 'Task A', NULL, 'Pending', 'User')
        "#,
    )
    .bind(task_a)
    .bind(feature_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO tasks (id, feature_id, title, description, status, created_by)
        VALUES ($1, $2, 'Task B', NULL, 'Pending', 'User')
        "#,
    )
    .bind(task_b)
    .bind(feature_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("INSERT INTO task_labels (task_id, label_id) VALUES ($1, $2)")
        .bind(task_a)
        .bind(label_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("INSERT INTO task_labels (task_id, label_id) VALUES ($1, $2)")
        .bind(task_b)
        .bind(label_id)
        .execute(&mut *tx)
        .await?;

    let duplicate_pair = sqlx::query("INSERT INTO task_labels (task_id, label_id) VALUES ($1, $2)")
        .bind(task_a)
        .bind(label_id)
        .execute(&mut *tx)
        .await;
    assert!(duplicate_pair.is_err());

    sqlx::query("DELETE FROM labels WHERE id = $1")
        .bind(label_id)
        .execute(&mut *tx)
        .await?;

    let remaining_links: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM task_labels WHERE task_id IN ($1, $2)")
        .bind(task_a)
        .bind(task_b)
        .fetch_one(&mut *tx)
        .await?;
    assert_eq!(remaining_links, 0);

    let remaining_tasks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tasks WHERE id IN ($1, $2)")
        .bind(task_a)
        .bind(task_b)
        .fetch_one(&mut *tx)
        .await?;
    assert_eq!(remaining_tasks, 2);

    tx.rollback().await?;
    Ok(())
}

#[tokio::test]
async fn task_labels_reject_cross_scope_association() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };
    sqlx::migrate!("./migrations").run(&pool).await?;

    let mut tx = pool.begin().await?;
    let (_company_id, _user_id, _project_id, feature_id) = seed_company_workspace(&mut tx).await?;

    let personal_user_id = Uuid::new_v4();
    let personal_label_id = Uuid::new_v4();
    let task_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO users (id, fullname, email, password_hash, account_type, role)
        VALUES ($1, 'Personal User', $2, 'unused', 'personal', NULL)
        "#,
    )
    .bind(personal_user_id)
    .bind(format!("personal-{}@example.test", Uuid::new_v4()))
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO labels (id, name, color, company_id, user_id)
        VALUES ($1, 'Private', '#778899', NULL, $2)
        "#,
    )
    .bind(personal_label_id)
    .bind(personal_user_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO tasks (id, feature_id, title, description, status, created_by)
        VALUES ($1, $2, 'Task with wrong label', NULL, 'Pending', 'User')
        "#,
    )
    .bind(task_id)
    .bind(feature_id)
    .execute(&mut *tx)
    .await?;

    let cross_scope_insert = sqlx::query(
        r#"
        INSERT INTO task_labels (task_id, label_id)
        SELECT $1, l.id
        FROM labels l
        INNER JOIN tasks t ON t.id = $1
        INNER JOIN features f ON f.id = t.feature_id
        INNER JOIN projects p ON p.id = f.project_id
        WHERE l.id = $2
          AND (
            (p.company_id IS NOT NULL AND l.company_id = p.company_id)
            OR
            (p.company_id IS NULL AND l.user_id = p.owner_user_id)
          )
        "#,
    )
    .bind(task_id)
    .bind(personal_label_id)
    .execute(&mut *tx)
    .await?;
    assert_eq!(cross_scope_insert.rows_affected(), 0);

    tx.rollback().await?;
    Ok(())
}
