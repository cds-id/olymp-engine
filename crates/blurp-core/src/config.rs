use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct BlurpConfig {
    pub app: AppConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub auth: AuthConfig,
    pub payment: PaymentConfig,
    pub shipping: ShippingConfig,
    pub email: EmailConfig,
    pub storage: StorageConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub url: String,
    pub environment: Env,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Env {
    Dev,
    Staging,
    Production,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_access_ttl_secs: u64,
    pub jwt_refresh_ttl_secs: u64,
    pub magic_link_ttl_secs: u64,
    pub cookie_secret: String,
    pub rate_limit_per_email: u32,
    pub rate_limit_per_ip: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaymentConfig {
    pub provider: PaymentProviderType,
    pub api_key: String,
    pub secret_key: String,
    pub webhook_secret: String,
    pub payment_expiry_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentProviderType {
    Xendit,
    Midtrans,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShippingConfig {
    pub provider: ShippingProviderType,
    pub api_key: String,
    pub origin_district_id: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShippingProviderType {
    Rajaongkir,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailConfig {
    pub provider: EmailProviderType,
    pub api_key: String,
    pub from: String,
    pub domain: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmailProviderType {
    Sendgrid,
    Ses,
    Mailgun,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub cdn_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub bind_addr: String,
}

impl BlurpConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        dotenvy::from_filename(".env.local").ok();
        dotenvy::dotenv().ok();

        let cfg = config::Config::builder()
            .add_source(config::File::with_name("config/default").required(false))
            .add_source(config::File::with_name("config/production").required(false))
            .add_source(config::Environment::with_prefix("BLURP").separator("__"))
            .build()?;

        cfg.try_deserialize()
    }
}
