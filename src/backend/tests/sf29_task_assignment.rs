use sqlx::PgPool;
use uuid::Uuid;

async fn connect_pool() -> Option<PgPool> {
    let database_url = std::env::var("DATABASE_URL").ok()?;
    PgPool::connect(&database_url).await.ok()
}

#[tokio::test]
async fn task_assignee_can_be_set_and_cleared() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };
    sqlx::migrate!("./migrations").run(&pool).await?;

    let mut tx = pool.begin().await?;
    let company_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let feature_id = Uuid::new_v4();
    let task_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO companies (id, name, company_email, country, timezone, status)
        VALUES ($1, 'SF29 Co', $2, 'TR', 'UTC', 'active')
        "#,
    )
    .bind(company_id)
    .bind(format!("sf29-assignee-{}@example.test", Uuid::new_v4()))
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO users (id, fullname, email, password_hash, account_type, role)
        VALUES ($1, 'Member', $2, 'unused', 'company', 'contributor')
        "#,
    )
    .bind(user_id)
    .bind(format!("member-{}@example.test", Uuid::new_v4()))
    .execute(&mut *tx)
    .await?;

    sqlx::query("INSERT INTO company_users (company_id, user_id) VALUES ($1, $2)")
        .bind(company_id)
        .bind(user_id)
        .execute(&mut *tx)
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
    .execute(&mut *tx)
    .await?;

    sqlx::query("INSERT INTO project_memberships (project_id, user_id, role) VALUES ($1, $2, 'contributor')")
        .bind(project_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("INSERT INTO features (id, project_id, title, status) VALUES ($1, $2, 'Feature', 'Pending')")
        .bind(feature_id)
        .bind(project_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO tasks (id, feature_id, title, description, status, created_by, priority, assignee_user_id, creator_user_id)
        VALUES ($1, $2, 'Task', NULL, 'Pending', 'User', 'Standard', NULL, $3)
        "#,
    )
    .bind(task_id)
    .bind(feature_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE tasks SET assignee_user_id = $2 WHERE id = $1")
        .bind(task_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    let assigned: Option<Uuid> = sqlx::query_scalar("SELECT assignee_user_id FROM tasks WHERE id = $1")
        .bind(task_id)
        .fetch_one(&mut *tx)
        .await?;
    assert_eq!(assigned, Some(user_id));

    sqlx::query("UPDATE tasks SET assignee_user_id = NULL WHERE id = $1")
        .bind(task_id)
        .execute(&mut *tx)
        .await?;
    let cleared: Option<Uuid> = sqlx::query_scalar("SELECT assignee_user_id FROM tasks WHERE id = $1")
        .bind(task_id)
        .fetch_one(&mut *tx)
        .await?;
    assert_eq!(cleared, None);

    tx.rollback().await?;
    Ok(())
}

#[tokio::test]
async fn task_assignee_change_event_can_be_persisted() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };
    sqlx::migrate!("./migrations").run(&pool).await?;

    let actor = serde_json::json!({
        "kind": "user",
        "user_id": Uuid::new_v4(),
        "company_id": Uuid::new_v4(),
    });
    let payload = serde_json::json!({
        "entity_type": "task",
        "entity_id": Uuid::new_v4(),
        "project_id": Uuid::new_v4(),
        "feature_id": Uuid::new_v4(),
        "old_assignee_user_id": serde_json::Value::Null,
        "new_assignee_user_id": Uuid::new_v4(),
    });

    sqlx::query(
        r#"
        INSERT INTO audit_events (
            ts_ms, event_type, actor, payload, request_id, ip_address, user_agent
        ) VALUES ($1, 'task_assignee_changed', $2, $3, $4, NULL, 'test')
        "#,
    )
    .bind(chrono::Utc::now().timestamp_millis())
    .bind(actor)
    .bind(payload)
    .bind(Uuid::new_v4())
    .execute(&pool)
    .await?;

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_events WHERE event_type = 'task_assignee_changed'",
    )
    .fetch_one(&pool)
    .await?;
    assert!(count >= 1);
    Ok(())
}
