use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{District, Province};
use olymp_core::AppError;

pub struct RegionRepository;

impl RegionRepository {
    // ─── Provinces ───

    pub async fn list_provinces(pool: &PgPool) -> Result<Vec<Province>, AppError> {
        sqlx::query_as::<_, Province>("SELECT * FROM provinces ORDER BY name")
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)
    }

    pub async fn get_province(pool: &PgPool, id: Uuid) -> Result<Option<Province>, AppError> {
        sqlx::query_as::<_, Province>("SELECT * FROM provinces WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)
    }

    pub async fn create_province(pool: &PgPool, name: &str) -> Result<Province, AppError> {
        let slug = slugify(name);
        sqlx::query_as::<_, Province>(
            "INSERT INTO provinces (name, slug) VALUES ($1, $2) RETURNING *",
        )
        .bind(name)
        .bind(&slug)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    // ─── Districts ───

    pub async fn list_districts(
        pool: &PgPool,
        province_id: Uuid,
    ) -> Result<Vec<District>, AppError> {
        sqlx::query_as::<_, District>(
            "SELECT * FROM districts WHERE province_id = $1 ORDER BY name",
        )
        .bind(province_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn get_district(pool: &PgPool, id: Uuid) -> Result<Option<District>, AppError> {
        sqlx::query_as::<_, District>("SELECT * FROM districts WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)
    }

    pub async fn create_district(
        pool: &PgPool,
        province_id: Uuid,
        name: &str,
    ) -> Result<District, AppError> {
        let slug = slugify(name);
        sqlx::query_as::<_, District>(
            "INSERT INTO districts (province_id, name, slug) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(province_id)
        .bind(name)
        .bind(&slug)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
