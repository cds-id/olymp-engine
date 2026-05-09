use crate::email::EmailProvider;
use crate::models::EmailMessage;
use async_trait::async_trait;
use blurp_core::error::AppError;
use lettre::{
    message::{header::ContentType, Mailbox, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

pub struct SmtpProvider {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from_address: String,
    from_name: String,
}

impl SmtpProvider {
    pub fn new(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        from_address: String,
        from_name: String,
        use_starttls: bool,
    ) -> Result<Self, AppError> {
        let creds = Credentials::new(username.to_string(), password.to_string());

        let transport = if use_starttls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)
                .map_err(|e| AppError::Internal(format!("SMTP config error: {}", e)))?
                .port(port)
                .credentials(creds)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::relay(host)
                .map_err(|e| AppError::Internal(format!("SMTP config error: {}", e)))?
                .port(port)
                .credentials(creds)
                .build()
        };

        Ok(Self {
            transport,
            from_address,
            from_name,
        })
    }

    /// Create from environment variables (Lark SMTP)
    pub fn from_env() -> Result<Self, AppError> {
        let host = dotenvy::var("BLURP__EMAIL__SMTP_HOST")
            .unwrap_or_else(|_| "smtp.larksuite.com".to_string());
        let port: u16 = dotenvy::var("BLURP__EMAIL__SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse()
            .unwrap_or(587);
        let username = dotenvy::var("BLURP__EMAIL__SMTP_USER")
            .map_err(|_| AppError::Internal("BLURP__EMAIL__SMTP_USER not set".into()))?;
        let password = dotenvy::var("BLURP__EMAIL__SMTP_PASSWORD")
            .map_err(|_| AppError::Internal("BLURP__EMAIL__SMTP_PASSWORD not set".into()))?;
        let from_address = dotenvy::var("BLURP__EMAIL__SMTP_FROM")
            .unwrap_or_else(|_| "no-reply@ciptadusa.com".to_string());
        let from_name = dotenvy::var("BLURP__EMAIL__FROM_NAME")
            .unwrap_or_else(|_| "SoraStore".to_string());
        let use_starttls = port == 587;

        Self::new(&host, port, &username, &password, from_address, from_name, use_starttls)
    }
}

#[async_trait]
impl EmailProvider for SmtpProvider {
    async fn send(&self, message: EmailMessage) -> Result<(), AppError> {
        let from_mailbox: Mailbox = format!("{} <{}>", self.from_name, self.from_address)
            .parse()
            .map_err(|e| AppError::Internal(format!("Invalid from address: {}", e)))?;

        let to_mailbox: Mailbox = message.to.parse()
            .map_err(|e| AppError::Internal(format!("Invalid to address: {}", e)))?;

        let email_builder = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(&message.subject);

        let email = if let Some(ref text) = message.text {
            email_builder
                .multipart(
                    MultiPart::alternative()
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_PLAIN)
                                .body(text.clone())
                        )
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_HTML)
                                .body(message.html.clone())
                        )
                )
                .map_err(|e| AppError::Internal(format!("Email build error: {}", e)))?
        } else {
            email_builder
                .header(ContentType::TEXT_HTML)
                .body(message.html.clone())
                .map_err(|e| AppError::Internal(format!("Email build error: {}", e)))?
        };

        self.transport
            .send(email)
            .await
            .map_err(|e| AppError::Internal(format!("SMTP send error: {}", e)))?;

        tracing::info!("Email sent via SMTP to {}", message.to);
        Ok(())
    }

    fn from_address(&self) -> &str {
        &self.from_address
    }

    fn from_name(&self) -> &str {
        &self.from_name
    }
}
