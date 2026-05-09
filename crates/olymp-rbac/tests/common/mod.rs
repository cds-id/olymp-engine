use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::path::Path;

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

        let db = TestDb { pool };
        db.run_migrations().await;
        db
    }

    async fn run_migrations(&self) {
        // Run migrations in dependency order
        let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let crates_dir = crate_root.parent().unwrap(); // crates/

        let migration_dirs = [
            "olymp-auth/migrations",
            "olymp-region/migrations",
            "olymp-event/migrations",
            "olymp-rbac/migrations",
            "olymp-participant/migrations",
        ];

        for dir in &migration_dirs {
            let migration_path = crates_dir.join(dir);
            let canonical = migration_path
                .canonicalize()
                .unwrap_or_else(|e| panic!("Migration path {:?} not found: {}", migration_path, e));

            let mut files: Vec<_> = std::fs::read_dir(&canonical)
                .unwrap_or_else(|e| panic!("Cannot read {:?}: {}", canonical, e))
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "sql"))
                .collect();
            files.sort_by_key(|e| e.file_name());

            for entry in files {
                let sql = std::fs::read_to_string(entry.path())
                    .unwrap_or_else(|e| panic!("Cannot read {:?}: {}", entry.path(), e));
                // Execute entire file as one statement
                match sqlx::raw_sql(&sql).execute(&self.pool).await {
                    Ok(_) => {}
                    Err(e) => {
                        let msg = e.to_string();
                        if msg.contains("already exists")
                            || msg.contains("duplicate key")
                        {
                            continue;
                        }
                        panic!(
                            "Migration failed on {:?}\nError: {}",
                            entry.path(),
                            e
                        );
                    }
                }
            }
        }
    }

    pub async fn cleanup(&self) {
        let tables = [
            "assignment_permission_overrides",
            "user_role_assignments",
            "role_permissions",
            "roles",
            "permissions",
        ];
        for table in &tables {
            sqlx::query(&format!("TRUNCATE TABLE {} CASCADE", table))
                .execute(&self.pool)
                .await
                .ok();
        }
        // Re-seed roles + permissions
        self.run_seeds().await;
    }

    async fn run_seeds(&self) {
        let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let seeds = [
            "migrations/002_seed_permissions.sql",
            "migrations/003_seed_roles.sql",
        ];
        for seed in &seeds {
            let path = crate_root.join(seed).canonicalize().expect("seed path");
            let sql = std::fs::read_to_string(&path).expect("read seed");
            sqlx::query(&sql)
                .execute(&self.pool)
                .await
                .unwrap_or_else(|e| panic!("Seed {:?} failed: {}", path, e));
        }
    }

    pub async fn create_test_user(&self, email_prefix: &str) -> uuid::Uuid {
        let id = uuid::Uuid::now_v7();
        let email = format!("{}+{}@test.com", email_prefix, &id.to_string()[..8]);
        sqlx::query("INSERT INTO auth.users (id, email, name, is_guest) VALUES ($1, $2, $3, false)")
            .bind(id)
            .bind(&email)
            .bind("Test User")
            .execute(&self.pool)
            .await
            .expect("create test user");
        id
    }
}
