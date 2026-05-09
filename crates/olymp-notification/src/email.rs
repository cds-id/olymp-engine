use crate::models::*;
use olymp_core::error::AppError;
use async_trait::async_trait;

#[async_trait]
pub trait EmailProvider: Send + Sync {
    async fn send(&self, message: EmailMessage) -> Result<(), AppError>;
    fn from_address(&self) -> &str;
    fn from_name(&self) -> &str;
}

pub struct EmailService {
    provider: Box<dyn EmailProvider>,
}

impl EmailService {
    pub fn new(provider: Box<dyn EmailProvider>) -> Self {
        Self { provider }
    }

    pub async fn send_registration(&self, to: &str, data: RegistrationData) -> Result<(), AppError> {
        let html = crate::templates::registration_html(&data);
        let text = crate::templates::registration_text(&data);

        self.provider.send(EmailMessage {
            to: to.to_string(),
            subject: "Verify your Olymp LMS account".to_string(),
            html,
            text: Some(text),
        }).await
    }

    pub async fn send_welcome(&self, to: &str, data: WelcomeData) -> Result<(), AppError> {
        let html = crate::templates::welcome_html(&data);
        let text = crate::templates::welcome_text(&data);

        self.provider.send(EmailMessage {
            to: to.to_string(),
            subject: "Welcome to Olymp LMS!".to_string(),
            html,
            text: Some(text),
        }).await
    }

    pub async fn send_magic_link(&self, to: &str, data: MagicLinkData) -> Result<(), AppError> {
        let html = crate::templates::magic_link_html(&data);
        let text = crate::templates::magic_link_text(&data);

        self.provider.send(EmailMessage {
            to: to.to_string(),
            subject: "Your login link for Olymp LMS".to_string(),
            html,
            text: Some(text),
        }).await
    }

    pub async fn send_password_reset(&self, to: &str, data: PasswordResetData) -> Result<(), AppError> {
        let html = crate::templates::password_reset_html(&data);
        let text = crate::templates::password_reset_text(&data);

        self.provider.send(EmailMessage {
            to: to.to_string(),
            subject: "Reset your Olymp LMS password".to_string(),
            html,
            text: Some(text),
        }).await
    }
}
