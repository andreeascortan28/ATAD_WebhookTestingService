use serde::{Serialize, Deserialize};
use sqlx::FromRow;

/// Represents a stored webhook request
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
    pub content_type: Option<String>,
    pub forward_url: Option<String>,
}

impl WebhookConfig {
    pub fn new_default() -> Self {
        Self {
            webhook_id: "".to_string(),
            status_code: Some(200),
            response_body: Some("OK".to_string()),
            content_type: Some("text/plain".to_string()),
            forward_url: None,
        }
    }
}

/// Represents a newly created webhook
#[derive(Serialize, Debug, Clone)]
pub struct NewWebhook {
    pub id: String,
    pub webhook_url: String,
}

/// Represents an event broadcasted to WebSocket clients
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebhookEvent {
    pub webhook_id: String,
    pub request_id: String,
    pub method: String,
    pub headers: String,
    pub body: String,
    pub query: String,
    pub created_at: String,
}

impl From<StoredRequest> for WebhookEvent {
    fn from(req: StoredRequest) -> Self {
        Self {
            webhook_id: req.webhook_id,
            request_id: req.id,
            method: req.method,
            headers: req.headers,
            body: req.body,
            query: req.query,
            created_at: req.created_at,
        }
    }
}