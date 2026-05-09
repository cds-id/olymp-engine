use blurp_core::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserAddress {
    pub id: Uuid,
    pub user_id: Uuid,
    pub label: Option<String>,
    pub name: String,
    pub phone: String,
    pub street: String,
    pub city: String,
    pub province: String,
    pub postal_code: String,
    pub country: String,
    pub district_id: i32,
    pub is_default: bool,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateAddressRequest {
    pub label: Option<String>,
    pub name: String,
    pub phone: String,
    pub street: String,
    pub city: String,
    pub province: String,
    pub postal_code: String,
    pub country: Option<String>,
    pub district_id: i32,
    pub is_default: Option<bool>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateAddressRequest {
    pub label: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub province: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub district_id: Option<i32>,
}

pub struct AddressService {
    pool: PgPool,
}

impl AddressService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, user_id: Uuid) -> Result<Vec<UserAddress>, AppError> {
        sqlx::query_as::<_, UserAddress>(
            "SELECT * FROM auth.user_addresses WHERE user_id = $1 ORDER BY is_default DESC, created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn get(&self, user_id: Uuid, id: Uuid) -> Result<UserAddress, AppError> {
        sqlx::query_as::<_, UserAddress>(
            "SELECT * FROM auth.user_addresses WHERE id = $1 AND user_id = $2"
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("Address not found".to_string()))
    }

    pub async fn create(&self, user_id: Uuid, req: CreateAddressRequest) -> Result<UserAddress, AppError> {
        let id = Uuid::now_v7();
        let is_default = req.is_default.unwrap_or(false);
        let country = req.country.unwrap_or_else(|| "Indonesia".to_string());

        if is_default {
            self.unset_defaults(user_id).await?;
        }

        sqlx::query(
            "INSERT INTO auth.user_addresses (id, user_id, label, name, phone, street, city, province, postal_code, country, district_id, is_default)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"
        )
        .bind(id).bind(user_id).bind(&req.label).bind(&req.name).bind(&req.phone)
        .bind(&req.street).bind(&req.city).bind(&req.province).bind(&req.postal_code)
        .bind(&country).bind(req.district_id).bind(is_default)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        self.get(user_id, id).await
    }

    pub async fn update(&self, user_id: Uuid, id: Uuid, req: UpdateAddressRequest) -> Result<UserAddress, AppError> {
        self.get(user_id, id).await?;

        sqlx::query(
            "UPDATE auth.user_addresses SET
               label = COALESCE($1, label),
               name = COALESCE($2, name),
               phone = COALESCE($3, phone),
               street = COALESCE($4, street),
               city = COALESCE($5, city),
               province = COALESCE($6, province),
               postal_code = COALESCE($7, postal_code),
               country = COALESCE($8, country),
               district_id = COALESCE($9, district_id),
               updated_at = now()
             WHERE id = $10 AND user_id = $11"
        )
        .bind(&req.label).bind(&req.name).bind(&req.phone)
        .bind(&req.street).bind(&req.city).bind(&req.province)
        .bind(&req.postal_code).bind(&req.country).bind(req.district_id)
        .bind(id).bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        self.get(user_id, id).await
    }

    pub async fn delete(&self, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let rows = sqlx::query(
            "DELETE FROM auth.user_addresses WHERE id = $1 AND user_id = $2"
        )
        .bind(id).bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?
        .rows_affected();

        if rows == 0 {
            return Err(AppError::NotFound("Address not found".to_string()));
        }
        Ok(())
    }

    pub async fn set_default(&self, user_id: Uuid, id: Uuid) -> Result<UserAddress, AppError> {
        self.get(user_id, id).await?;
        self.unset_defaults(user_id).await?;

        sqlx::query(
            "UPDATE auth.user_addresses SET is_default = true, updated_at = now() WHERE id = $1 AND user_id = $2"
        )
        .bind(id).bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        self.get(user_id, id).await
    }

    async fn unset_defaults(&self, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE auth.user_addresses SET is_default = false WHERE user_id = $1 AND is_default = true"
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;
        Ok(())
    }
}
