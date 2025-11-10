use axum::{
    extract::{Path, State, Query},
    response::IntoResponse,
    Json,
};
use axum::body::Bytes;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use crate::AppState;
use crate::models::{StoredRequest, WebhookConfig, WebhookEvent};
use crate::utils;
use crate::routes::ws;

#[derive(Serialize)]
pub struct NewWebhookResponse {
    pub id: String,
    pub webhook_url: String,
    pub dashboard_url: String,
}

pub async fn create_webhook(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();

    if let Err(err) = state.create_webhook(&id).await {
        eprintln!("Error creating webhook: {err}");
    }

    let webhook_url = format!("/webhook/{}", id);
    let dashboard_url = format!("/dashboard/{}", id);

    Json(NewWebhookResponse {
        id,
        webhook_url,
        dashboard_url,
    })
}

pub async fn webhook_handler(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashMap<String, String>>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let headers_map = utils::headers_to_map(&headers);
    let req_id = Uuid::new_v4().to_string();

    // Create the StoredRequest
    let stored_req = StoredRequest {
        id: req_id.clone(),
        webhook_id: id.clone(),
        method: "POST".to_string(),
        headers: serde_json::to_string(&headers_map).unwrap_or_default(),
        body: String::from_utf8(body.to_vec()).unwrap_or_default(),
        query: serde_json::to_string(&query).unwrap_or_default(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    // Save request to the database
    if let Err(err) = state.db.store_request(&stored_req).await {
        eprintln!("DB store error: {err}");
    }

    // Wrap the request in a WebhookEvent and broadcast
    let event: WebhookEvent = stored_req.clone().into();
    if let Err(err) = state.tx.send(event) {
        eprintln!("Broadcast error: {err}");
        // Fallback to per-webhook WS broadcast if needed
        ws::broadcast_to_clients(&id, &stored_req).await;
    }

    // Get custom response config
    let config = state.db.get_response_config(&id).await.unwrap_or_default();

    // Optional forwarding
    if let Some(forward_url) = &config.forward_url {
        if let Err(err) = utils::forward_request(forward_url, &stored_req).await {
            eprintln!("Forwarding error: {err}");
        }
    }

    // Build response
    let status = StatusCode::from_u16(config.status_code.unwrap_or(200))
        .unwrap_or(StatusCode::OK);

    (status, config.response_body.unwrap_or_else(|| "OK".to_string()))
}

#[derive(Deserialize)]
pub struct CustomResponsePayload {
    status_code: Option<u16>,
    response_body: Option<String>,
    forward_url: Option<String>,
}

pub async fn set_custom_response(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CustomResponsePayload>,
) -> impl IntoResponse {
    let config = WebhookConfig {
        webhook_id: id.clone(),
        status_code: payload.status_code,
        response_body: payload.response_body,
        forward_url: payload.forward_url,
    };

    if let Err(err) = state.set_response_config(&config).await {
        eprintln!("Error setting response config: {err}");
        return axum::http::StatusCode::INTERNAL_SERVER_ERROR;
    }

    axum::http::StatusCode::OK
}
