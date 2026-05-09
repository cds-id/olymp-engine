use crate::email::EmailProvider;
use crate::models::EmailMessage;
use async_trait::async_trait;
use blurp_core::error::AppError;

pub struct MailgunProvider {
    api_key: String,
    domain: String,
    from_address: String,
    from_name: String,
    http_client: reqwest::Client,
}

impl MailgunProvider {
    pub fn new(api_key: String, domain: String, from_name: String) -> Self {
        let from_address = format!("postmaster@{}", domain);
        Self {
            api_key,
            domain,
            from_address,
            from_name,
            http_client: reqwest::Client::new(),
        }
    }

    pub fn from_env() -> Result<Self, AppError> {
        let api_key = dotenvy::var("BLURP__EMAIL__MAILGUN_API_KEY")
            .map_err(|_| AppError::Internal("BLURP__EMAIL__MAILGUN_API_KEY not set".into()))?;
        let domain = dotenvy::var("BLURP__EMAIL__MAILGUN_DOMAIN")
            .unwrap_or_else(|_| "mail.infomedialink.com".to_string());
        let from_name = dotenvy::var("BLURP__EMAIL__FROM_NAME")
            .unwrap_or_else(|_| "SoraStore".to_string());
        
        Ok(Self::new(api_key, domain, from_name))
    }
}

#[async_trait]
impl EmailProvider for MailgunProvider {
    async fn send(&self, message: EmailMessage) -> Result<(), AppError> {
        let url = format!("https://api.mailgun.net/v3/{}/messages", self.domain);
        
        let mut form = vec![
            ("from", format!("{} <{}>", self.from_name, self.from_address)),
            ("to", message.to.clone()),
            ("subject", message.subject.clone()),
            ("html", message.html.clone()),
        ];
        
        if let Some(text) = &message.text {
            form.push(("text", text.clone()));
        }

        let response = self.http_client
            .post(&url)
            .basic_auth("api", Some(&self.api_key))
            .form(&form)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Mailgun request failed: {}", e)))?;

        if response.status().is_success() {
            tracing::info!("Email sent via Mailgun to {}", message.to);
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!("Mailgun error {}: {}", status, body);
            Err(AppError::Internal(format!("Mailgun error: {} - {}", status, body)))
        }
    }

    fn from_address(&self) -> &str {
        &self.from_address
    }

    fn from_name(&self) -> &str {
        &self.from_name
    }
}
