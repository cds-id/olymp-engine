use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub to: String,
    pub subject: String,
    pub html: String,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailType {
    Registration,
    Welcome,
    MagicLink,
    PasswordReset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationData {
    pub name: String,
    pub verification_link: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WelcomeData {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicLinkData {
    pub name: String,
    pub magic_link: String,
    pub expires_in_minutes: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetData {
    pub name: String,
    pub reset_link: String,
    pub expires_in_minutes: i32,
}
