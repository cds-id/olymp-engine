use async_trait::async_trait;
use bytes::Bytes;
use std::time::Duration;

use crate::error::AppError;

pub type AppResult<T> = Result<T, AppError>;

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
