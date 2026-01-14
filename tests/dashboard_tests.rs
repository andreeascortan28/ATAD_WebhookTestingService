mod common;

use axum::{Router, routing::get};
use tower::ServiceExt;
use axum::http::Request;
use webhook_tester::routes::dashboard::dashboard_handler;
use common::test_state;

#[tokio::test]
async fn dashboard_renders_html() {
    let state = test_state().await;

    let app = Router::new()
        .route("/dashboard/:id", get(dashboard_handler))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/dashboard/test")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("Webhook Dashboard"));
}
