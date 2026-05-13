use sqlx::PgPool;
use uuid::Uuid;

async fn connect_pool() -> Option<PgPool> {
    let database_url = std::env::var("DATABASE_URL").ok()?;
    PgPool::connect(&database_url).await.ok()
}

#[tokio::test]
async fn company_and_personal_projects_are_isolated() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = connect_pool().await else {
        return Ok(());
    };

    sqlx::migrate!("./migrations").run(&pool).await?;

    let mut tx = pool.begin().await?;

    let company_a_id = Uuid::new_v4();
    let company_b_id = Uuid::new_v4();
    let company_a_user_id = Uuid::new_v4();
    let company_b_user_id = Uuid::new_v4();
    let personal_user_id = Uuid::new_v4();
    let project_a_id = Uuid::new_v4();
    let project_b_id = Uuid::new_v4();
    let project_personal_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO companies (id, name, company_email, country, timezone, status)
        VALUES ($1, $2, $3, 'TR', 'UTC', 'active')
        "#,
    )
    .bind(company_a_id)
    .bind("Company A")
    .bind(format!("a-{}@example.test", Uuid::new_v4()))
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO companies (id, name, company_email, country, timezone, status)
        VALUES ($1, $2, $3, 'TR', 'UTC', 'active')
        "#,
    )
    .bind(company_b_id)
    .bind("Company B")
    .bind(format!("b-{}@example.test", Uuid::new_v4()))
    .execute(&mut *tx)
    .await?;

    for (id, email, account_type, role) in [
        (
            company_a_user_id,
            format!("company-a-{}@example.test", Uuid::new_v4()),
            "company",
            Some("company_admin"),
        ),
        (
            company_b_user_id,
            format!("company-b-{}@example.test", Uuid::new_v4()),
            "company",
            Some("company_admin"),
        ),
        (
            personal_user_id,
            format!("personal-{}@example.test", Uuid::new_v4()),
            "personal",
            None,
        ),
    ] {
        sqlx::query(
            r#"
            INSERT INTO users (id, fullname, email, password_hash, account_type, role)
            VALUES ($1, 'Test User', $2, 'not-used', $3, $4)
            "#,
        )
        .bind(id)
        .bind(email)
        .bind(account_type)
        .bind(role)
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query("INSERT INTO company_users (company_id, user_id) VALUES ($1, $2)")
        .bind(company_a_id)
        .bind(company_a_user_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("INSERT INTO company_users (company_id, user_id) VALUES ($1, $2)")
        .bind(company_b_id)
        .bind(company_b_user_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO projects (
            id,
            owner_user_id,
            company_id,
            created_by_user_id,
            name,
            status
        ) VALUES ($1, NULL, $2, $3, 'Company A Project', 'Pending')
        "#,
    )
    .bind(project_a_id)
    .bind(company_a_id)
    .bind(company_a_user_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO projects (
            id,
            owner_user_id,
            company_id,
            created_by_user_id,
            name,
            status
        ) VALUES ($1, NULL, $2, $3, 'Company B Project', 'Pending')
        "#,
    )
    .bind(project_b_id)
    .bind(company_b_id)
    .bind(company_b_user_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO projects (
            id,
            owner_user_id,
            company_id,
            created_by_user_id,
            name,
            status
        ) VALUES ($1, $2, NULL, $3, 'Personal Project', 'Pending')
        "#,
    )
    .bind(project_personal_id)
    .bind(personal_user_id)
    .bind(personal_user_id)
    .execute(&mut *tx)
    .await?;

    let company_a_cannot_read_b: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT id
        FROM projects
        WHERE id = $1
          AND (
            ($2::uuid IS NOT NULL AND company_id = $2)
            OR ($3::uuid IS NOT NULL AND owner_user_id = $3 AND company_id IS NULL)
          )
        "#,
    )
    .bind(project_b_id)
    .bind(company_a_id)
    .bind(Option::<Uuid>::None)
    .fetch_optional(&mut *tx)
    .await?;
    assert!(company_a_cannot_read_b.is_none());

    let personal_c_cannot_read_a: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT id
        FROM projects
        WHERE id = $1
          AND (
            ($2::uuid IS NOT NULL AND company_id = $2)
            OR ($3::uuid IS NOT NULL AND owner_user_id = $3 AND company_id IS NULL)
          )
        "#,
    )
    .bind(project_a_id)
    .bind(Option::<Uuid>::None)
    .bind(personal_user_id)
    .fetch_optional(&mut *tx)
    .await?;
    assert!(personal_c_cannot_read_a.is_none());

    let company_a_visible_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM projects
        WHERE
            ($1::uuid IS NOT NULL AND company_id = $1)
            OR ($2::uuid IS NOT NULL AND owner_user_id = $2 AND company_id IS NULL)
        "#,
    )
    .bind(company_a_id)
    .bind(Option::<Uuid>::None)
    .fetch_one(&mut *tx)
    .await?;
    assert_eq!(company_a_visible_count, 1);

    tx.rollback().await?;
    Ok(())
}
