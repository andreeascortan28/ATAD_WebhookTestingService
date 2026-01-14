mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use webhook_tester::routes::webhook::webhook_handler;
use webhook_tester::models::WebhookConfig;
use common::test_state;

#[tokio::test]
async fn webhook_stores_request_and_returns_default_response() {
    let state = test_state().await;

    let app = axum::Router::new()
        .route("/webhook/:id", axum::routing::post(webhook_handler))
        .with_state(state.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/webhook/test123")
                .method("POST")
                .body(Body::from("hello"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let stored = state.db.get_request(
        &sqlx::query_scalar::<_, String>("SELECT id FROM requests LIMIT 1")
            .fetch_one(&state.db.pool)
            .await
            .unwrap()
    ).await.unwrap();

    assert_eq!(stored.body, "hello");
}

#[tokio::test]
async fn webhook_respects_custom_response() {
    let state = test_state().await;

    state.db
        .set_response_config(&WebhookConfig {
            webhook_id: "abc".into(),
            status_code: Some(201),
            response_body: Some("Created".into()),
            content_type: Some("text/plain".into()),
            forward_url: None,
        })
        .await
        .unwrap();

    let app = axum::Router::new()
        .route("/webhook/:id", axum::routing::post(webhook_handler))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/webhook/abc")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}
