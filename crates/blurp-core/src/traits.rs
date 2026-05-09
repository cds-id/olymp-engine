use async_trait::async_trait;
use axum::http::HeaderMap;
use bytes::Bytes;
use chrono::NaiveDate;
use std::time::Duration;

use crate::error::AppError;

pub type AppResult<T> = Result<T, AppError>;

// ─── Payment Provider ───

#[derive(Debug, Clone)]
pub struct CreateInvoiceRequest {
    pub order_id: uuid::Uuid,
    pub amount: i64,
    pub currency: String,
    pub description: String,
    pub customer_email: String,
    pub payment_methods: Vec<String>,
    pub expiry_secs: u64,
}

#[derive(Debug, Clone)]
pub struct Invoice {
    pub external_id: String,
    pub payment_url: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct WebhookEvent {
    pub external_id: String,
    pub event_type: String,
    pub status: PaymentStatus,
    pub amount: i64,
    pub method: Option<String>,
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Paid,
    Failed,
    Expired,
    Refunded,
    Settled,
}

#[derive(Debug, Clone)]
pub struct Settlement {
    pub external_id: String,
    pub amount: i64,
    pub settled_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
pub trait PaymentProvider: Send + Sync {
    async fn create_invoice(&self, req: CreateInvoiceRequest) -> AppResult<Invoice>;
    async fn verify_webhook(&self, headers: &HeaderMap, body: &[u8]) -> AppResult<WebhookEvent>;
    async fn check_status(&self, external_id: &str) -> AppResult<PaymentStatus>;
    async fn get_settlement_report(&self, date: NaiveDate) -> AppResult<Vec<Settlement>>;
}

// ─── Shipping Provider ───

#[derive(Debug, Clone)]
pub struct ShippingRateRequest {
    pub origin_postal_code: String,
    pub destination_postal_code: String,
    pub weight_grams: u32,
    pub couriers: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ShippingRate {
    pub courier: String,
    pub service: String,
    pub cost: i64,
    pub etd: String,
}

#[async_trait]
pub trait ShippingProvider: Send + Sync {
    async fn get_rates(&self, req: ShippingRateRequest) -> AppResult<Vec<ShippingRate>>;
}

// ─── Email Provider ───

#[derive(Debug, Clone)]
pub struct EmailMessage {
    pub to: String,
    pub subject: String,
    pub html_body: String,
    pub text_body: Option<String>,
}

#[async_trait]
pub trait EmailProvider: Send + Sync {
    async fn send(&self, msg: EmailMessage) -> AppResult<()>;
}

// ─── Storage Provider ───

#[async_trait]
pub trait StorageProvider: Send + Sync {
    async fn upload(&self, key: &str, data: Bytes, content_type: &str) -> AppResult<String>;
    async fn delete(&self, key: &str) -> AppResult<()>;
    async fn presigned_url(&self, key: &str, expires: Duration) -> AppResult<String>;
}
