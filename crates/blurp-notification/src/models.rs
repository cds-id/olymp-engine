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
    OrderConfirmation,
    OrderShipped,
    OrderDelivered,
    GuestOrderTracking,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderConfirmationData {
    pub name: String,
    pub order_number: String,
    pub total_idr: i64,
    pub items: Vec<OrderItemData>,
    pub shipping_address: String,
    pub tracking_link: Option<String>,  // For guest orders
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItemData {
    pub product_name: String,
    pub variant_name: String,
    pub quantity: i32,
    pub price_idr: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderShippedData {
    pub name: String,
    pub order_number: String,
    pub courier: String,
    pub tracking_number: String,
    pub tracking_link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestTrackingData {
    pub name: String,
    pub order_number: String,
    pub tracking_link: String,
}
