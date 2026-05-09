use blurp_core::config::BlurpConfig;
use blurp_notification::EmailService;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: redis::Client,
    pub config: Arc<BlurpConfig>,
    pub email: Option<Arc<EmailService>>,
}
