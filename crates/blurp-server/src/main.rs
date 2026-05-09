mod openapi;

use blurp_auth::handlers::*;
use blurp_core::BlurpConfig;
use sqlx::postgres::PgPoolOptions;
use blurp_auth::AppState;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    tracing::info!("Starting Olympiad LMS Engine");

    let config = BlurpConfig::load()?;

    tracing::info!("Connecting to database...");
    let db = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await?;

    tracing::info!("Database pool created");

    let redis = redis::Client::open(config.redis.url.clone())?;
    tracing::info!("Connected to Redis");

    // Setup email service (try Mailgun first, fall back to SMTP)
    let email_service: Option<Arc<blurp_notification::EmailService>> = {
        use blurp_notification::{EmailService, MailgunProvider, SmtpProvider};

        if let Ok(provider) = MailgunProvider::from_env() {
            tracing::info!("Email: Mailgun configured");
            Some(Arc::new(EmailService::new(Box::new(provider))))
        } else if let Ok(provider) = SmtpProvider::from_env() {
            tracing::info!("Email: SMTP configured");
            Some(Arc::new(EmailService::new(Box::new(provider))))
        } else {
            tracing::warn!("Email: No provider configured");
            None
        }
    };

    let state = AppState {
        db,
        redis,
        config: Arc::new(config.clone()),
        email: email_service,
    };

    // Start background workers
    blurp_worker::start_workers(state.db.clone(), state.redis.clone());

    let mut app = axum::Router::new()
        .route("/health", axum::routing::get(|| async { axum::Json(serde_json::json!({"status": "ok"})) }))
        // Auth endpoints
        .route("/api/auth/register", axum::routing::post(register))
        .route("/api/auth/login", axum::routing::post(login))
        .route("/api/auth/logout", axum::routing::post(logout))
        .route("/api/auth/refresh", axum::routing::post(refresh))
        .route("/api/auth/magic-link", axum::routing::post(request_magic_link))
        .route("/api/auth/callback", axum::routing::post(magic_link_callback))
        // User endpoints
        .route("/api/users/me", axum::routing::get(me).put(update_me))
        .route("/api/users/me/password", axum::routing::post(change_password))
        // Participant endpoints (stub)
        .route("/api/participant/register", axum::routing::post(|| async { axum::Json(serde_json::json!({"status": "not_implemented"})) }))
        .route("/api/participant/profile", axum::routing::get(|| async { axum::Json(serde_json::json!({"status": "not_implemented"})) }))
        // Exam endpoints (stub)
        .route("/api/exam/list", axum::routing::get(|| async { axum::Json(serde_json::json!({"status": "not_implemented"})) }))
        .route("/api/exam/:id/start", axum::routing::post(|| async { axum::Json(serde_json::json!({"status": "not_implemented"})) }))
        // Ranking endpoints (stub)
        .route("/api/ranking/leaderboard", axum::routing::get(|| async { axum::Json(serde_json::json!({"status": "not_implemented"})) }))
        // Monitoring endpoints (stub)
        .route("/api/monitoring/exam-progress", axum::routing::get(|| async { axum::Json(serde_json::json!({"status": "not_implemented"})) }))
        .with_state(state);

    // Add Swagger UI only in development
    if matches!(config.app.environment, blurp_core::config::Env::Dev) {
        use utoipa::OpenApi;
        use utoipa_swagger_ui::SwaggerUi;
        
        tracing::info!("Swagger UI enabled at /swagger-ui");
        app = app.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()));
    }

    // CORS
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(
            config.app.url.parse::<axum::http::HeaderValue>()
                .map(tower_http::cors::AllowOrigin::exact)
                .unwrap_or_else(|_| tower_http::cors::AllowOrigin::any())
        )
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::PATCH,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600));

    let app = app.layer(cors);

    let listener = TcpListener::bind(&config.server.bind_addr).await?;
    tracing::info!("Listening on {} (CORS origin: {})", config.server.bind_addr, config.app.url);

    axum::serve(listener, app).await?;

    Ok(())
}
