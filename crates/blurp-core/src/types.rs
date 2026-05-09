use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Money {
    pub amount: i64,
    pub currency: Currency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    Idr,
}

impl Default for Currency {
    fn default() -> Self {
        Self::Idr
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub name: String,
    pub phone: String,
    pub street: String,
    pub district: String,
    pub city: String,
    pub province: String,
    pub postal_code: String,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct PaginationParams {
    #[validate(range(min = 1, max = 100))]
    pub per_page: Option<u32>,
    pub cursor: Option<String>,
}

impl PaginationParams {
    pub fn per_page_or_default(&self) -> u32 {
        self.per_page.unwrap_or(20)
    }
}
