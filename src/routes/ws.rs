use axum::{
    extract::{Path, ws::{WebSocketUpgrade, WebSocket, Message}},
    response::IntoResponse,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use serde_json;
use crate::models::StoredRequest;
use lazy_static::lazy_static;

lazy_static! {
    static ref BROADCASTERS: Arc<Mutex<HashMap<String, broadcast::Sender<StoredRequest>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

pub async fn ws_handler(
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(id.clone(), socket))
}

async fn handle_socket(webhook_id: String, mut socket: WebSocket) {
    // Get or create a broadcast channel for this webhook
    let tx = {
        let mut broadcasters = BROADCASTERS.lock().await;
        broadcasters
            .entry(webhook_id.clone())
            .or_insert_with(|| {
                let (tx, _rx) = broadcast::channel(100);
                tx
            })
            .clone()
    };

    // Subscribe to the channel
    let mut rx = tx.subscribe();

    // Notify client
    let _ = socket.send(Message::Text("Connected to WebSocket".into())).await;

    // Forward messages from channel to this client
    while let Ok(request) = rx.recv().await {
        let msg = serde_json::to_string(&request).unwrap_or_else(|_| "{}".to_string());
        if socket.send(Message::Text(msg)).await.is_err() {
            // Client disconnected, stop the loop
            break;
        }
    }
}

pub async fn broadcast_to_clients(webhook_id: &str, request: &StoredRequest) {
    let broadcasters = BROADCASTERS.lock().await;
    if let Some(sender) = broadcasters.get(webhook_id) {
        let _ = sender.send(request.clone()); // ignore error if no clients
    }
}
