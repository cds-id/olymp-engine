use blurp_core::error::AppError;
use reqwest::Client;

pub struct MailgunEmailProvider {
    api_key: String,
    domain: String,
    from: String,
    client: Client,
}

impl MailgunEmailProvider {
    pub fn new(api_key: String, domain: String, from: String) -> Self {
        Self {
            api_key,
            domain,
            from,
            client: Client::new(),
        }
    }

    pub async fn send_magic_link(&self, email: &str, link: &str) -> Result<(), AppError> {
        let url = format!("https://api.mailgun.net/v3/{}/messages", self.domain);

        let params = [
            ("from", self.from.as_str()),
            ("to", email),
            ("subject", "Your SoraStore Login Link"),
            (
                "html",
                &format!(
                    "<p>Click <a href=\"{}\">here</a> to login to SoraStore.</p><p>Link expires in 15 minutes.</p>",
                    link
                ),
            ),
        ];

        self.client
            .post(&url)
            .basic_auth("api", Some(&self.api_key))
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .error_for_status()
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }
}
