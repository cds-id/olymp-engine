use crate::models::*;
use blurp_core::error::AppError;
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
            subject: "Verify your SoraStore account".to_string(),
            html,
            text: Some(text),
        }).await
    }

    pub async fn send_welcome(&self, to: &str, data: WelcomeData) -> Result<(), AppError> {
        let html = crate::templates::welcome_html(&data);
        let text = crate::templates::welcome_text(&data);
        
        self.provider.send(EmailMessage {
            to: to.to_string(),
            subject: "Welcome to SoraStore!".to_string(),
            html,
            text: Some(text),
        }).await
    }

    pub async fn send_magic_link(&self, to: &str, data: MagicLinkData) -> Result<(), AppError> {
        let html = crate::templates::magic_link_html(&data);
        let text = crate::templates::magic_link_text(&data);
        
        self.provider.send(EmailMessage {
            to: to.to_string(),
            subject: "Your login link for SoraStore".to_string(),
            html,
            text: Some(text),
        }).await
    }

    pub async fn send_password_reset(&self, to: &str, data: PasswordResetData) -> Result<(), AppError> {
        let html = crate::templates::password_reset_html(&data);
        let text = crate::templates::password_reset_text(&data);
        
        self.provider.send(EmailMessage {
            to: to.to_string(),
            subject: "Reset your SoraStore password".to_string(),
            html,
            text: Some(text),
        }).await
    }

    pub async fn send_order_confirmation(&self, to: &str, data: OrderConfirmationData) -> Result<(), AppError> {
        let html = crate::templates::order_confirmation_html(&data);
        let text = crate::templates::order_confirmation_text(&data);
        
        self.provider.send(EmailMessage {
            to: to.to_string(),
            subject: format!("Order Confirmed - {}", data.order_number),
            html,
            text: Some(text),
        }).await
    }

    pub async fn send_order_shipped(&self, to: &str, data: OrderShippedData) -> Result<(), AppError> {
        let html = crate::templates::order_shipped_html(&data);
        let text = crate::templates::order_shipped_text(&data);
        
        self.provider.send(EmailMessage {
            to: to.to_string(),
            subject: format!("Your order {} has shipped!", data.order_number),
            html,
            text: Some(text),
        }).await
    }

    pub async fn send_guest_tracking(&self, to: &str, data: GuestTrackingData) -> Result<(), AppError> {
        let html = crate::templates::guest_tracking_html(&data);
        let text = crate::templates::guest_tracking_text(&data);
        
        self.provider.send(EmailMessage {
            to: to.to_string(),
            subject: format!("Track your order - {}", data.order_number),
            html,
            text: Some(text),
        }).await
    }
}
