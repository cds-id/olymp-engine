pub mod address;
pub mod email;
pub mod error;
pub mod notifications;
pub mod jwt;
pub mod magic_link;
pub mod user;
pub mod password;
pub mod oauth;
pub mod registration;
pub mod extractors;
pub mod handlers;
pub mod state;

pub use jwt::{Claims, JwtService};
pub use magic_link::MagicLinkService;
pub use user::UserService;
pub use password::PasswordService;
pub use oauth::OAuthService;
pub use registration::RegistrationService;
pub use state::AppState;

pub async fn migrate(pool: &sqlx::PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
