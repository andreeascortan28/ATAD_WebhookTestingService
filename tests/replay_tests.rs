use axum::{
    routing::post,
    Router,
    http::StatusCode,
    body::{Body, to_bytes},
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

use webhook_tester::replay::{replay_request, MOCK_FORWARD_REQUEST};
use webhook_tester::models::{StoredRequest, WebhookEvent};
use webhook_tester::{AppState};
use webhook_tester::utils::new_for_tests;

use tower::ServiceExt;

fn test_stored_request() -> StoredRequest {
    StoredRequest {
        id: "req-1".into(),
        webhook_id: "wh-1".into(),
        method: "POST".into(),
        headers: r#"{"x-test":"123"}"#.into(),
        body: "hello world".into(),
        query: "".into(),
        created_at: "2025-01-01T00:00:00Z".into(),
    }
}

fn test_app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/replay/:id", post(replay_request))
        .with_state(state)
}

async fn read_body(body: Body) -> String {
    let bytes = to_bytes(body, 1024 * 1024).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test]
async fn replay_fails_when_target_missing() {
    let (tx, _rx) = broadcast::channel::<WebhookEvent>(100);
    let db = new_for_tests().await;

    let state = Arc::new(AppState { db, tx });
    let app = test_app(state);

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/replay/req-1")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "target": "" }).to_string()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    let body_str = read_body(response.into_body()).await;

    assert!(body_str.contains("Missing target URL"));
}

#[tokio::test]
async fn replay_fails_when_request_not_found() {
    let (tx, _rx) = broadcast::channel::<WebhookEvent>(100);
    let db = new_for_tests().await;

    let state = Arc::new(AppState { db, tx });
    let app = test_app(state);

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/replay/req-404")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "target": "http://example.com" }).to_string()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    let body_str = read_body(response.into_body()).await;

    assert!(body_str.contains("Request not found"));
}

#[tokio::test]
async fn replay_sends_request_successfully() {
    use std::future::Future;
    use std::pin::Pin;

    let (tx, _rx) = broadcast::channel::<WebhookEvent>(100);
    let db = new_for_tests().await;
    db.store_request(&test_stored_request()).await.unwrap();

    let state = Arc::new(AppState { db, tx });
    let app = test_app(state);

    let captured: Arc<Mutex<Option<(String, String)>>> = Arc::new(Mutex::new(None));
    let captured_clone = captured.clone();

    let mock_fn: Box<
        dyn Fn(&str, &StoredRequest) -> Pin<Box<dyn Future<Output = Result<(), anyhow::Error>> + Send>>
        + Send
        + Sync,
    > = Box::new(move |url: &str, req: &StoredRequest| {
        let captured_inner = captured_clone.clone();
        let url_owned = url.to_string();
        let body_owned = req.body.clone();

        Box::pin(async move {
            let mut lock = captured_inner.lock().await;
            *lock = Some((url_owned, body_owned));
            Ok(())
        })
    });

    MOCK_FORWARD_REQUEST.set(mock_fn).ok();

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/replay/req-1")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "target": "http://mock.url" }).to_string()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let lock = captured.lock().await;
    let (url, body) = lock.as_ref().unwrap();
    assert_eq!(url, "http://mock.url");
    assert_eq!(body, "hello world");
}
