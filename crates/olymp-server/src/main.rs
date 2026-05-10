mod middleware;
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
        .route("/api/users/me/roles", axum::routing::get(my_roles))
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
        pool: db_pool.clone(),
        redis: redis_client.clone(),
    };
    let rbac_routes = axum::Router::new()
        .route("/api/rbac/roles", axum::routing::get(olymp_rbac::handlers::list_roles).post(olymp_rbac::handlers::create_role))
        .route("/api/rbac/roles/{role_id}", axum::routing::put(olymp_rbac::handlers::update_role))
        .route("/api/rbac/permissions", axum::routing::get(olymp_rbac::handlers::list_permissions))
        .route("/api/rbac/roles/{role_id}/permissions", axum::routing::get(olymp_rbac::handlers::get_role_permissions).post(olymp_rbac::handlers::assign_role_permissions))
        .route("/api/rbac/assignments", axum::routing::get(olymp_rbac::handlers::list_assignments).post(olymp_rbac::handlers::create_assignment))
        .route("/api/rbac/assignments/{id}", axum::routing::put(olymp_rbac::handlers::update_assignment).delete(olymp_rbac::handlers::delete_assignment))
        .with_state(rbac_state);

    // Participant routes (State<PgPool>)
    let participant_routes = axum::Router::new()
        .route("/api/users/me/participations", axum::routing::get(olymp_participant::handlers::my_participations))
        .route("/api/events/{event_id}/participants", axum::routing::get(olymp_participant::handlers::list_event_participants).post(olymp_participant::handlers::register_participant))
        .route("/api/participants/{id}", axum::routing::get(olymp_participant::handlers::get_participant).put(olymp_participant::handlers::update_participant))
        .route("/api/participants/{id}/verify", axum::routing::post(olymp_participant::handlers::verify_participant))
        .route("/api/participants/{id}/approve", axum::routing::post(olymp_participant::handlers::approve_participant))
        .route("/api/participants/{id}/reject", axum::routing::post(olymp_participant::handlers::reject_participant))
        .route("/api/stages/{stage_id}/participants", axum::routing::get(olymp_participant::handlers::list_stage_participants))
        .with_state(db_pool.clone());

    // Exam routes (State<PgPool>)
    let exam_routes = axum::Router::new()
        .route("/api/users/me/sessions", axum::routing::get(olymp_exam::handlers::my_sessions))
        .route("/api/stages/{stage_id}/exams", axum::routing::get(olymp_exam::handlers::list_exams).post(olymp_exam::handlers::create_exam))
        .route("/api/exams/{exam_id}", axum::routing::get(olymp_exam::handlers::get_exam).put(olymp_exam::handlers::update_exam))
        .route("/api/exams/{exam_id}/questions", axum::routing::get(olymp_exam::handlers::list_questions).post(olymp_exam::handlers::create_question))
        .route("/api/exams/{exam_id}/questions/{question_id}", axum::routing::put(olymp_exam::handlers::update_question).delete(olymp_exam::handlers::delete_question))
        .route("/api/exams/{exam_id}/sessions", axum::routing::post(olymp_exam::handlers::assign_session))
        .route("/api/sessions/{session_id}", axum::routing::get(olymp_exam::handlers::get_session))
        .route("/api/sessions/{session_id}/start", axum::routing::post(olymp_exam::handlers::start_session))
        .route("/api/sessions/{session_id}/answers", axum::routing::get(olymp_exam::handlers::list_answers).post(olymp_exam::handlers::save_answer))
        .route("/api/sessions/{session_id}/submit", axum::routing::post(olymp_exam::handlers::submit_session))
        .with_state(db_pool.clone());

    // Monitoring routes (State<MonitoringState>)
    let (monitor_tx, _) = tokio::sync::broadcast::channel::<olymp_monitoring::models::MonitorEvent>(256);
    let monitoring_state = olymp_monitoring::handlers::MonitoringState {
        pool: db_pool.clone(),
        tx: monitor_tx,
    };
    let monitoring_routes = axum::Router::new()
        .route("/api/cheating-logs", axum::routing::post(olymp_monitoring::handlers::create_cheating_log))
        .route("/api/sessions/{session_id}/cheating-logs", axum::routing::get(olymp_monitoring::handlers::list_cheating_logs))
        .route("/api/sessions/{session_id}/progress", axum::routing::get(olymp_monitoring::handlers::get_progress).put(olymp_monitoring::handlers::update_progress))
        .route("/api/exams/{exam_id}/progress", axum::routing::get(olymp_monitoring::handlers::list_exam_progress))
        .route("/api/exams/{exam_id}/monitor/stream", axum::routing::get(olymp_monitoring::handlers::monitor_stream))
        .route("/api/audit-logs", axum::routing::get(olymp_monitoring::handlers::query_audit_logs))
        .with_state(monitoring_state);

    // Ranking routes (State<PgPool>)
    let ranking_routes = axum::Router::new()
        .route("/api/stages/{stage_id}/ranking/rules", axum::routing::get(olymp_ranking::handlers::get_ranking_rule).post(olymp_ranking::handlers::upsert_ranking_rule))
        .route("/api/stages/{stage_id}/ranking/calculate", axum::routing::post(olymp_ranking::handlers::calculate_ranking))
        .route("/api/stages/{stage_id}/ranking", axum::routing::get(olymp_ranking::handlers::get_ranking))
        .route("/api/stages/{stage_id}/ranking/review", axum::routing::post(olymp_ranking::handlers::review_ranking))
        .route("/api/stages/{stage_id}/ranking/approve", axum::routing::post(olymp_ranking::handlers::approve_ranking))
        .route("/api/stages/{stage_id}/ranking/publish", axum::routing::post(olymp_ranking::handlers::publish_ranking))
        .route("/api/stages/{stage_id}/promote", axum::routing::post(olymp_ranking::handlers::promote))
        .with_state(db_pool.clone());

    // Certificate routes (State<PgPool>)
    let certificate_routes = axum::Router::new()
        .route("/api/events/{event_id}/certificates/templates", axum::routing::get(olymp_certificate::handlers::list_templates).post(olymp_certificate::handlers::create_template))
        .route("/api/certificates/templates/{template_id}", axum::routing::get(olymp_certificate::handlers::get_template).put(olymp_certificate::handlers::update_template))
        .route("/api/stages/{stage_id}/certificates/generate", axum::routing::post(olymp_certificate::handlers::generate_certificates))
        .route("/api/participants/{participant_id}/certificates", axum::routing::get(olymp_certificate::handlers::list_participant_certificates))
        .route("/api/certificates/{certificate_id}", axum::routing::get(olymp_certificate::handlers::get_certificate))
        .route("/api/events/{event_id}/finalize", axum::routing::post(olymp_certificate::handlers::finalize_event))
        .with_state(db_pool.clone());

    app = app.merge(region_routes).merge(event_routes).merge(rbac_routes).merge(participant_routes).merge(exam_routes).merge(monitoring_routes).merge(ranking_routes).merge(certificate_routes);

    // Add Swagger UI only in development
    if matches!(config.app.environment, olymp_core::config::Env::Dev) {
        use utoipa::OpenApi;
        use utoipa_swagger_ui::SwaggerUi;
        
        tracing::info!("Swagger UI enabled at /swagger-ui");
        app = app.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()));
    }

    // Auth middleware
    let auth_state = middleware::AuthState {
        db: db_pool.clone(),
        redis: redis_client.clone(),
        jwt_secret: config.auth.jwt_secret.clone(),
        jwt_access_ttl_secs: config.auth.jwt_access_ttl_secs,
    };

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

    let app = app
        .layer(axum::middleware::from_fn_with_state(auth_state, middleware::auth_middleware))
        .layer(cors);

    let listener = TcpListener::bind(&config.server.bind_addr).await?;
    tracing::info!("Listening on {} (CORS origin: {})", config.server.bind_addr, config.app.url);

    axum::serve(listener, app).await?;

    Ok(())
}
