use axum::{
    routing::get,
    Router,
};
use futures_util::StreamExt;
use tokio::time::{timeout, Duration};
use tokio_tungstenite::tungstenite::Message;

use webhook_tester::routes::ws::{ws_handler, broadcast_to_clients};
use webhook_tester::models::StoredRequest;

fn test_app() -> Router {
    Router::new().route("/ws/:id", get(ws_handler))
}

fn test_stored_request(webhook_id: &str, id: &str, body: &str) -> StoredRequest {
    StoredRequest {
        id: id.to_string(),
        webhook_id: webhook_id.to_string(),
        method: "POST".to_string(),
        headers: "{}".to_string(),
        body: body.to_string(),
        query: "".to_string(),
        created_at: "2025-01-01T00:00:00Z".to_string(),
    }
}

async fn spawn_app() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap();

    let addr = listener.local_addr().unwrap();
    let app = test_app();

    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .unwrap();
    });

    format!("ws://{}", addr)
}


#[tokio::test]
async fn websocket_sends_connected_message() {
    let base = spawn_app().await;
    let url = format!("{}/ws/test-webhook", base);

    let (mut ws, _) = tokio_tungstenite::connect_async(url)
        .await
        .expect("websocket connect failed");

    let msg = timeout(Duration::from_secs(1), ws.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();

    let Message::Text(text) = msg else {
        panic!("expected text message");
    };

    assert!(text.contains("\"status\":\"connected\""));
    assert!(text.contains("test-webhook"));
}

#[tokio::test]
async fn broadcast_reaches_connected_client() {
    let base = spawn_app().await;
    let url = format!("{}/ws/abc123", base);

    let (mut ws, _) = tokio_tungstenite::connect_async(url).await.unwrap();

    // Drain "connected" message
    let _ = ws.next().await;

    let req = test_stored_request("abc123", "req-1", "hello");

    broadcast_to_clients("abc123", &req).await;

    let msg = timeout(Duration::from_secs(1), ws.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();

    let Message::Text(text) = msg else {
        panic!("expected text message");
    };

    let received: StoredRequest = serde_json::from_str(&text).unwrap();
    assert_eq!(received.id, "req-1");
    assert_eq!(received.body, "hello");
}

#[tokio::test]
async fn broadcast_with_no_clients_does_not_panic() {
    let req = test_stored_request("none", "req-none", "no listeners");

    broadcast_to_clients("non-existent", &req).await;
}
