pub mod email;
pub mod mailgun;
pub mod smtp;
pub mod templates;
pub mod models;

pub use email::{EmailProvider, EmailService};
pub use mailgun::MailgunProvider;
pub use smtp::SmtpProvider;
pub use models::{OrderShippedData, OrderConfirmationData, GuestTrackingData};
