use utoipa::OpenApi;
use olymp_auth::handlers::*;
use olymp_auth::notifications::UpdateNotificationPreferences;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Olymp Engine API",
        version = "0.1.0",
        description = "Olympiad LMS backend API built with Rust and Axum",
    ),
    paths(
        // Auth
        olymp_auth::handlers::request_magic_link,
        olymp_auth::handlers::magic_link_callback,
        olymp_auth::handlers::register,
        olymp_auth::handlers::login,
        olymp_auth::handlers::logout,
        olymp_auth::handlers::refresh,
        olymp_auth::handlers::forgot_password,
        olymp_auth::handlers::reset_password,
        olymp_auth::handlers::me,
        olymp_auth::handlers::update_me,
        olymp_auth::handlers::change_password,
        olymp_auth::handlers::get_notifications,
        olymp_auth::handlers::update_notifications,
        // Regions
        olymp_region::handlers::list_provinces,
        olymp_region::handlers::get_province,
        olymp_region::handlers::create_province,
        olymp_region::handlers::list_districts,
        olymp_region::handlers::create_district,
        // Events
        olymp_event::handlers::list_education_levels,
        olymp_event::handlers::create_education_level,
        olymp_event::handlers::list_subjects,
        olymp_event::handlers::create_subject,
        olymp_event::handlers::list_events,
        olymp_event::handlers::get_event,
        olymp_event::handlers::create_event,
        olymp_event::handlers::update_event,
        olymp_event::handlers::list_stages,
        olymp_event::handlers::create_stage,
        olymp_event::handlers::update_stage_status,
        olymp_event::handlers::list_event_categories,
        olymp_event::handlers::create_event_category,
    ),
    components(
        schemas(
            // Auth
            MagicLinkRequest,
            MagicLinkResponse,
            CallbackRequest,
            AuthResponse,
            RefreshRequest,
            RegisterRequest,
            LoginRequest,
            LogoutRequest,
            ChangePasswordRequest,
            ForgotPasswordRequest,
            ResetPasswordRequest,
            UpdateProfileRequest,
            UpdateNotificationPreferences,
            // Region
            olymp_region::models::Province,
            olymp_region::models::District,
            olymp_region::models::CreateProvinceRequest,
            olymp_region::models::CreateDistrictRequest,
            // Event
            olymp_event::models::EducationLevel,
            olymp_event::models::Subject,
            olymp_event::models::Event,
            olymp_event::models::Stage,
            olymp_event::models::EventCategory,
            olymp_event::models::CreateEducationLevelRequest,
            olymp_event::models::CreateSubjectRequest,
            olymp_event::models::CreateEventRequest,
            olymp_event::models::UpdateEventRequest,
            olymp_event::models::CreateStageRequest,
            olymp_event::models::UpdateStageStatusRequest,
            olymp_event::models::CreateEventCategoryRequest,
            // Core enums
            olymp_core::types::Tier,
            olymp_core::types::EventStatus,
            olymp_core::types::StageStatus,
        )
    ),
    tags(
        (name = "auth", description = "Authentication"),
        (name = "events", description = "Olympiad event management"),
        (name = "regions", description = "Region management"),
        (name = "participants", description = "Participant management"),
        (name = "exams", description = "Exam management"),
        (name = "ranking", description = "Ranking and qualification"),
        (name = "monitoring", description = "Exam monitoring"),
        (name = "rbac", description = "Role-based access control"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            );
        }
    }
}
