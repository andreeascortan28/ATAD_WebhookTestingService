use axum::http::HeaderMap;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use crate::models::StoredRequest;
use reqwest::Client;
use sqlx::SqlitePool;
use crate::db::Database;

pub fn headers_to_json(headers: &HeaderMap) -> Value {
    let mut map = serde_json::Map::new();
    for (k, v) in headers.iter() {
        map.insert(
            k.to_string(),
            Value::String(v.to_str().unwrap_or_default().to_string()),
        );
    }
    Value::Object(map)
}

pub fn headers_to_map(headers: &HeaderMap) -> HashMap<String, String> {
    headers
        .iter()
        .map(|(k, v)| {
            (
                k.to_string(),
                v.to_str().unwrap_or("").to_string()
            )
        })
        .collect()
}

/// Forward the webhook request to another URL
pub async fn forward_request(forward_url: &str, req: &StoredRequest) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let headers: HashMap<String, String> = serde_json::from_str(&req.headers).unwrap_or_default();
    let mut request_builder = client.post(forward_url).body(req.body.clone());

    for (key, value) in headers {
        request_builder = request_builder.header(&key, &value);
    }

    let _res = request_builder.send().await?;

    Ok(())
}

pub async fn new_for_tests() -> Arc<Database> {
    let pool = SqlitePool::connect(":memory:").await.unwrap();

    sqlx::query(
        r#"
        CREATE TABLE requests (
            id TEXT PRIMARY KEY,
            webhook_id TEXT NOT NULL,
            method TEXT NOT NULL,
            headers TEXT NOT NULL,
            body TEXT NOT NULL,
            query TEXT,
            created_at TEXT NOT NULL
        )
        "#,
    )
        .execute(&pool)
        .await
        .unwrap();

    Arc::new(Database { pool })
}

