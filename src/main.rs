use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use std::{net::SocketAddr, sync::Arc};
use tokio::fs;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let state = Arc::new(db::init_db().await?);

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
    println!("Create a new webhook at http://localhost:3000/new");

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

async fn home_page() -> axum::response::Html<String> {
    let html = match fs::read_to_string("static/index.html").await {
        Ok(content) => content,
        Err(_) => r#"
        <html></html>
        "#.to_string(),
    };
    axum::response::Html(html)
}
