pub mod routes;
pub mod db;
pub mod models;
pub mod replay;
pub mod utils;

use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<db::Database>,
    pub tx: broadcast::Sender<models::WebhookEvent>,
}

impl AppState {
    pub async fn create_webhook(&self, id: &str) -> anyhow::Result<()> {
        self.db.create_webhook(id).await?;
        Ok(())
    }

    pub async fn store_request(&self, req: &crate::models::StoredRequest) -> anyhow::Result<()> {
        self.db.store_request(req).await?;
        Ok(())
    }

    pub async fn get_response_config(
        &self,
        webhook_id: &str,
    ) -> Result<crate::models::WebhookConfig, sqlx::Error> {
        self.db.get_response_config(webhook_id).await
    }

    pub async fn set_response_config(
        &self,
        config: &crate::models::WebhookConfig,
    ) -> Result<(), sqlx::Error> {
        self.db.set_response_config(config).await
    }
}