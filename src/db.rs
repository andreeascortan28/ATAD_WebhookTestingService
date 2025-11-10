use sqlx::{SqlitePool, FromRow};
use anyhow::Result;
use chrono::Utc;
use std::{fs::OpenOptions, path::PathBuf};
use crate::models::{StoredRequest, WebhookConfig};

/// Represents the database connection layer.
#[derive(Clone)]
pub struct Database {
    pub pool: SqlitePool,
}

/// Initialize the SQLite database and return a Database instance.
pub async fn init_db() -> Result<Database> {
    let mut db_path: PathBuf = std::env::current_dir()?;
    db_path.push("webhooks.db");
    println!("Using database path: {}", db_path.display());

    if !db_path.exists() {
        OpenOptions::new()
            .write(true)
            .create(true)
            .open(&db_path)?;
    }

    let db_url = format!("sqlite://{}", db_path.to_string_lossy());
    let pool = SqlitePool::connect(&db_url).await?;

    sqlx::query("PRAGMA foreign_keys = ON;").execute(&pool).await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS requests (
            id TEXT PRIMARY KEY,
            webhook_id TEXT NOT NULL,
            method TEXT NOT NULL,
            headers TEXT,
            body TEXT,
            query TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )
        "#
    ).execute(&pool).await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS webhooks (
            id TEXT PRIMARY KEY,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )
        "#
    ).execute(&pool).await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS webhook_configs (
            webhook_id TEXT PRIMARY KEY,
            status_code INTEGER,
            response_body TEXT,
            forward_url TEXT
        )
        "#
    ).execute(&pool).await?;

    Ok(Database { pool })
}

impl Database {
    /// Insert a new webhook UUID.
    pub async fn create_webhook(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO webhooks (id, created_at) VALUES (?, ?)")
            .bind(id)
            .bind(Utc::now().to_rfc3339())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Store a webhook request.
    pub async fn store_request(&self, req: &StoredRequest) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO requests (id, webhook_id, method, headers, body, query, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
            .bind(&req.id)
            .bind(&req.webhook_id)
            .bind(&req.method)
            .bind(&req.headers)
            .bind(&req.body)
            .bind(&req.query)
            .bind(&req.created_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Retrieve a stored request by ID.
    pub async fn get_request(&self, req_id: &str) -> Result<StoredRequest, sqlx::Error> {
        sqlx::query_as::<_, StoredRequest>(
            "SELECT id, webhook_id, method, headers, body, query, created_at
             FROM requests WHERE id = ?1"
        )
            .bind(req_id)
            .fetch_one(&self.pool)
            .await
    }

    /// Save or update a custom response configuration.
    pub async fn set_response_config(&self, config: &WebhookConfig) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO webhook_configs (webhook_id, status_code, response_body, forward_url)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(webhook_id) DO UPDATE SET
                 status_code = excluded.status_code,
                 response_body = excluded.response_body,
                 forward_url = excluded.forward_url"
        )
            .bind(&config.webhook_id)
            .bind(config.status_code)
            .bind(&config.response_body)
            .bind(&config.forward_url)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Retrieve a webhook's custom response configuration.
    pub async fn get_response_config(&self, webhook_id: &str) -> Result<WebhookConfig, sqlx::Error> {
        let config = sqlx::query_as::<_, WebhookConfig>(
            "SELECT webhook_id, status_code, response_body, forward_url
             FROM webhook_configs WHERE webhook_id = ?"
        )
            .bind(webhook_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(config.unwrap_or_default())
    }
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            webhook_id: "".to_string(),
            status_code: Some(200),
            response_body: Some("OK".to_string()),
            forward_url: None,
        }
    }
}
