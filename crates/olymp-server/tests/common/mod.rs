use std::sync::Once;

static INIT: Once = Once::new();

pub fn init_test_env() {
    INIT.call_once(|| {
        dotenvy::from_filename(".env.local").ok();
    });
}

pub struct TestClient {
    pub client: reqwest::Client,
    pub base_url: String,
}

impl TestClient {
    pub fn new() -> Self {
        init_test_env();
        Self {
            client: reqwest::Client::new(),
            base_url: "http://localhost:8080".to_string(),
        }
    }

    pub async fn login(&self, username: &str, password: &str) -> String {
        let res = self.client
            .post(format!("{}/api/auth/login", self.base_url))
            .json(&serde_json::json!({
                "username": username,
                "password": password
            }))
            .send()
            .await
            .unwrap();
        
        let body: serde_json::Value = res.json().await.unwrap();
        body["data"]["access_token"].as_str().unwrap().to_string()
    }
}
