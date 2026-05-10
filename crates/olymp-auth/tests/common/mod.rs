use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool};

const TEST_DB_URL: &str = "postgres://blurp:blurp@localhost:5433/blurp";

pub struct TestDb {
    pub pool: PgPool,
}

impl TestDb {
    pub async fn new() -> Self {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(TEST_DB_URL)
            .await
            .expect("Failed to connect to test DB");

        // Run auth migrations
        Self::run_migrations(&pool).await;

        TestDb { pool }
    }

    async fn run_migrations(pool: &PgPool) {
        let migration_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
        let mut files: Vec<_> = std::fs::read_dir(&migration_dir)
            .expect("Failed to read migrations dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "sql").unwrap_or(false))
            .collect();
        files.sort_by_key(|e| e.file_name());

        for entry in files {
            let sql = std::fs::read_to_string(entry.path())
                .expect("Failed to read migration file");
            if let Err(e) = pool.execute(sqlx::raw_sql(&sql)).await {
                let msg = e.to_string();
                // Ignore "already exists" errors for idempotency
                if !msg.contains("already exists")
                    && !msg.contains("duplicate key")
                    && !msg.contains("does not exist")
                {
                    panic!("Migration {} failed: {}", entry.file_name().to_string_lossy(), msg);
                }
            }
        }
    }

    pub async fn cleanup(&self) {
        let tables = vec![
            "auth.notification_preferences",
            "auth.password_resets",
            "auth.oauth_providers",
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
