use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use std::{net::SocketAddr, sync::Arc};
use tokio::{fs, sync::broadcast};

mod routes;
mod db;
mod models;
mod replay;
mod utils;

use routes::{
    webhook::{create_webhook, webhook_handler, set_custom_response},
    dashboard::dashboard_handler,
    ws::ws_handler,
};

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize DB
    let db = Arc::new(db::init_db().await?);

    // create broadcast channel for real-time events
    let (tx, _rx) = broadcast::channel(100);

    let state = Arc::new(AppState { db, tx });

    let app = Router::new()
        .route("/", get(home_page))
        .route("/new", get(create_webhook))
        .route("/webhook/:id", post(webhook_handler))
        .route("/webhook/:id/config", post(set_custom_response))
        .route("/dashboard/:id", get(dashboard_handler))
        .route("/ws/:id", get(ws_handler))
        .route("/replay/:req_id", post(replay::replay_request))
        .layer(CorsLayer::very_permissive())
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running at http://localhost:3000");

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

async fn home_page() -> axum::response::Html<String> {
    let html = match fs::read_to_string("static/index.html").await {
        Ok(content) => content,
        Err(_) => r#"<html><body><h1>Webhook Service</h1></body></html>"#.to_string(),
    };
    axum::response::Html(html)
}
