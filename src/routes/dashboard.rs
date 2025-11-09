use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    Json,
};
use std::sync::Arc;
use crate::{db::AppState, models::StoredRequest};

pub async fn dashboard_handler(Path(id): Path<String>, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let requests = sqlx::query_as::<_, StoredRequest>(
        "SELECT * FROM requests WHERE webhook_id = ?1 ORDER BY created_at DESC"
    )
        .bind(id)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

    let html = std::fs::read_to_string("static/index.html").unwrap_or_else(|_| "<h1>Dashboard not found</h1>".to_string());
    Html(html)
}
