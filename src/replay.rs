use axum::{extract::{Path, State}, Json};
use serde_json::json;
use reqwest::Client;
use crate::{db::AppState, models::StoredRequest};
use std::sync::Arc;

pub async fn replay_request(
    Path(req_id): Path<i64>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let target_url = payload["target"].as_str().unwrap_or("");
    if target_url.is_empty() {
        return Json(json!({"error": "Missing target URL"}));
    }

    if let Ok(req) = sqlx::query_as::<_, StoredRequest>(
        "SELECT * FROM requests WHERE id = ?1"
    )
        .bind(req_id)
        .fetch_one(&state.pool)
        .await
    {
        let client = Client::new();
        let res = client.post(target_url)
            .body(req.body.clone())
            .send()
            .await;

        match res {
            Ok(_) => Json(json!({"status": "ok"})),
            Err(e) => Json(json!({"error": e.to_string()})),
        }
    } else {
        Json(json!({"error": "Request not found"}))
    }
}
