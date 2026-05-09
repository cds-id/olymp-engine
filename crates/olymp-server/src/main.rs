mod openapi;

use olymp_auth::handlers::*;
use olymp_core::OlympConfig;
use sqlx::postgres::PgPoolOptions;
use olymp_auth::AppState;
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

    let config = OlympConfig::load()?;

    tracing::info!("Connecting to database...");
    let db = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await?;

    tracing::info!("Database pool created");

    let redis = redis::Client::open(config.redis.url.clone())?;
    tracing::info!("Connected to Redis");

    // Setup email service (try Mailgun first, fall back to SMTP)
    let email_service: Option<Arc<olymp_notification::EmailService>> = {
        use olymp_notification::{EmailService, MailgunProvider, SmtpProvider};

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

    let db_pool = db.clone();
    let redis_client = redis.clone();

    let state = AppState {
        db,
        redis,
        config: Arc::new(config.clone()),
        email: email_service,
    };

    // Start background workers
    olymp_worker::start_workers(state.db.clone(), state.redis.clone());

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
        // TODO: Phase 2 — RBAC routes
        // TODO: Phase 3 — participant routes
        // TODO: Phase 4 — exam routes
        // TODO: Phase 5 — monitoring routes
        // TODO: Phase 6 — ranking + qualification routes
        .with_state(state);

    // Region routes (State<PgPool>)
    let region_routes = axum::Router::new()
        .route("/api/provinces", axum::routing::get(olymp_region::handlers::list_provinces).post(olymp_region::handlers::create_province))
        .route("/api/provinces/{id}", axum::routing::get(olymp_region::handlers::get_province))
        .route("/api/provinces/{province_id}/districts", axum::routing::get(olymp_region::handlers::list_districts).post(olymp_region::handlers::create_district))
        .with_state(db_pool.clone());

    // Event routes (State<PgPool>)
    let event_routes = axum::Router::new()
        .route("/api/education-levels", axum::routing::get(olymp_event::handlers::list_education_levels).post(olymp_event::handlers::create_education_level))
        .route("/api/subjects", axum::routing::get(olymp_event::handlers::list_subjects).post(olymp_event::handlers::create_subject))
        .route("/api/events", axum::routing::get(olymp_event::handlers::list_events).post(olymp_event::handlers::create_event))
        .route("/api/events/{id}", axum::routing::get(olymp_event::handlers::get_event).put(olymp_event::handlers::update_event))
        .route("/api/events/{event_id}/stages", axum::routing::get(olymp_event::handlers::list_stages).post(olymp_event::handlers::create_stage))
        .route("/api/stages/{id}/status", axum::routing::put(olymp_event::handlers::update_stage_status))
        .route("/api/events/{event_id}/categories", axum::routing::get(olymp_event::handlers::list_event_categories).post(olymp_event::handlers::create_event_category))
        .with_state(db_pool.clone());

    // RBAC routes (State<RbacState>)
    let rbac_state = olymp_rbac::handlers::RbacState {
        pool: db_pool,
        redis: redis_client,
    };
    let rbac_routes = axum::Router::new()
        .route("/api/rbac/roles", axum::routing::get(olymp_rbac::handlers::list_roles).post(olymp_rbac::handlers::create_role))
        .route("/api/rbac/roles/{role_id}", axum::routing::put(olymp_rbac::handlers::update_role))
        .route("/api/rbac/permissions", axum::routing::get(olymp_rbac::handlers::list_permissions))
        .route("/api/rbac/roles/{role_id}/permissions", axum::routing::get(olymp_rbac::handlers::get_role_permissions).post(olymp_rbac::handlers::assign_role_permissions))
        .route("/api/rbac/assignments", axum::routing::get(olymp_rbac::handlers::list_assignments).post(olymp_rbac::handlers::create_assignment))
        .route("/api/rbac/assignments/{id}", axum::routing::put(olymp_rbac::handlers::update_assignment).delete(olymp_rbac::handlers::delete_assignment))
        .with_state(rbac_state);

    app = app.merge(region_routes).merge(event_routes).merge(rbac_routes);

    // Add Swagger UI only in development
    if matches!(config.app.environment, olymp_core::config::Env::Dev) {
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
