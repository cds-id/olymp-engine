use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub struct TestDb {
    pub pool: PgPool,
}

impl TestDb {
    pub async fn new() -> Self {
        let url = "postgres://blurp:blurp@localhost:5433/blurp";

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await
            .expect("Failed to connect to test DB");

        TestDb { pool }
    }

    pub async fn cleanup(&self) {
        let tables = vec![
            "auth.magic_links",
            "auth.sessions",
            "auth.users",
        ];

        for table in tables {
            sqlx::query(&format!("TRUNCATE TABLE {} CASCADE", table))
                .execute(&self.pool)
                .await
                .ok();
        }
    }
}

pub async fn create_test_user(pool: &PgPool, email: &str) -> uuid::Uuid {
    let id = uuid::Uuid::now_v7();
    sqlx::query(
        "INSERT INTO auth.users (id, email, name, is_guest) VALUES ($1, $2, $3, false)"
    )
    .bind(id)
    .bind(email)
    .bind("Test User")
    .execute(pool)
    .await
    .expect("Failed to create test user");
    id
}
