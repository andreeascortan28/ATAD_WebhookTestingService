use axum::{
    extract::{
        Path,
        ws::{WebSocketUpgrade, WebSocket, Message},
    },
    response::IntoResponse,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use serde_json;
use crate::models::StoredRequest;
use lazy_static::lazy_static;

lazy_static! {
    // Global registry of broadcast senders, one per webhook ID
    static ref BROADCASTERS: Arc<Mutex<HashMap<String, broadcast::Sender<StoredRequest>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

pub async fn ws_handler(
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(id, socket))
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

    // Subscribe to the broadcast channel
    let mut rx = tx.subscribe();

    // Notify the client that the connection is established
    let _ = socket
        .send(Message::Text(format!(
            "{{\"status\":\"connected\",\"webhook_id\":\"{}\"}}",
            webhook_id
        )))
        .await;

    // Continuously send broadcast messages to this client
    while let Ok(request) = rx.recv().await {
        match serde_json::to_string(&request) {
            Ok(json) => {
                if socket.send(Message::Text(json)).await.is_err() {
                    // Client disconnected, break out of the loop
                    break;
                }
            }
            Err(err) => {
                eprintln!("Error serializing StoredRequest: {err}");
            }
        }
    }

    // Optionally: clean up empty broadcasters (optional optimization)
    {
        let mut broadcasters = BROADCASTERS.lock().await;
        if let Some(sender) = broadcasters.get(&webhook_id) {
            // If there are no active receivers, drop the channel
            if sender.receiver_count() == 0 {
                broadcasters.remove(&webhook_id);
            }
        }
    }
}

/// Broadcasts a stored webhook request to all active WebSocket clients
pub async fn broadcast_to_clients(webhook_id: &str, request: &StoredRequest) {
    let broadcasters = BROADCASTERS.lock().await;
    if let Some(sender) = broadcasters.get(webhook_id) {
        let _ = sender.send(request.clone()); // ignore if no active clients
    }
}
