use olymp_core::config::OlympConfig;
use olymp_notification::EmailService;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: redis::Client,
    pub config: Arc<OlympConfig>,
    pub email: Option<Arc<EmailService>>,
}
