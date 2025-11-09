use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Debug, Clone)]
pub struct StoredRequest {
    pub id: String,
    pub webhook_id: String,
    pub method: String,
    pub headers: String,
    pub body: String,
    pub query: String,
    pub created_at: String,
}

/// Webhook configuration / custom response
#[derive(Serialize, FromRow, Debug, Clone)]
pub struct WebhookConfig {
    pub webhook_id: String,
    pub status_code: Option<u16>,
    pub response_body: Option<String>,
    pub forward_url: Option<String>,
}

impl WebhookConfig {
    pub fn new_default() -> Self {
        Self {
            webhook_id: "".to_string(),
            status_code: Some(200),
            response_body: Some("OK".to_string()),
            forward_url: None,
        }
    }
}

#[derive(Serialize)]
pub struct NewWebhook {
    pub id: String,
    pub webhook_url: String,
}
