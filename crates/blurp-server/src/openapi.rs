use utoipa::OpenApi;
use blurp_auth::handlers::*;
use blurp_auth::address::{CreateAddressRequest, UpdateAddressRequest};
use blurp_auth::notifications::UpdateNotificationPreferences;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Blurp Engine API",
        version = "0.1.0",
        description = "E-commerce backend API built with Rust and Axum",
    ),
    paths(
        blurp_auth::handlers::request_magic_link,
        blurp_auth::handlers::magic_link_callback,
        blurp_auth::handlers::register,
        blurp_auth::handlers::login,
        blurp_auth::handlers::logout,
        blurp_auth::handlers::refresh,
        blurp_auth::handlers::forgot_password,
        blurp_auth::handlers::reset_password,
        blurp_auth::handlers::me,
        blurp_auth::handlers::update_me,
        blurp_auth::handlers::change_password,
        blurp_auth::handlers::list_addresses,
        blurp_auth::handlers::create_address,
        blurp_auth::handlers::get_address,
        blurp_auth::handlers::update_address,
        blurp_auth::handlers::delete_address,
        blurp_auth::handlers::set_default_address,
        blurp_auth::handlers::get_notifications,
        blurp_auth::handlers::update_notifications,
    ),
    components(
        schemas(
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
            CreateAddressRequest,
            UpdateAddressRequest,
            UpdateNotificationPreferences,
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "catalog", description = "Product catalog management"),
        (name = "cart", description = "Shopping cart operations"),
        (name = "wishlist", description = "Wishlist management"),
        (name = "orders", description = "Order processing"),
        (name = "shipping", description = "Shipping calculations"),
        (name = "payments", description = "Payment processing"),
        (name = "admin", description = "Admin operations"),
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
