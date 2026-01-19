use axum::{extract::{Path, State}, Json};
use serde_json::json;
use reqwest::{Client, header::{HeaderMap, HeaderName, HeaderValue}};
use std::sync::{Arc, OnceLock};
use std::collections::HashMap;

use crate::{AppState, models::StoredRequest};

#[derive(serde::Deserialize)]
pub struct ReplayPayload {
    target: String,
}

pub type ForwardRequestFn = dyn Fn(&str, &StoredRequest) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), anyhow::Error>> + Send>> + Send + Sync;

pub static MOCK_FORWARD_REQUEST: OnceLock<Box<ForwardRequestFn>> = OnceLock::new();

pub async fn replay_request(
    Path(req_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ReplayPayload>,
) -> Json<serde_json::Value> {
    let target_url = &payload.target;
    if target_url.is_empty() {
        return Json(json!({"error": "Missing target URL"}));
    }

    // Fetch the stored request
    let stored_req_result = state.db.get_request(&req_id).await;
    let stored_req: StoredRequest = match stored_req_result {
        Ok(req) => req,
        Err(_) => return Json(json!({"error": "Request not found"})),
    };

    // If the mock is set, call it
    if let Some(mock) = MOCK_FORWARD_REQUEST.get() {
        if let Err(e) = mock(target_url, &stored_req).await {
            return Json(json!({"error": e.to_string()}));
        }
        return Json(json!({"status": "ok"}));
    }

    // Convert headers JSON string -> HashMap -> HeaderMap
    let mut headers = HeaderMap::new();
    if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&stored_req.headers) {
        for (key, value) in map {
            if let Ok(name) = key.parse::<HeaderName>() {
                if let Ok(header_value) = value.parse::<HeaderValue>() {
                    headers.insert(name, header_value);
                }
                // else: skip invalid header value
            }
            // else: skip invalid header name
        }
    }

    // Send the request
    let client = Client::new();
    let res = client.post(target_url)
        .headers(headers)
        .body(stored_req.body.clone())
        .send()
        .await;

    match res {
        Ok(resp) => Json(json!({
            "status": "ok",
            "status_code": resp.status().as_u16()
        })),
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}
