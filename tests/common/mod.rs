use std::sync::Arc;
use tokio::sync::broadcast;
use webhook_tester::{AppState, db};

pub async fn test_state() -> Arc<AppState> {
    let pool = sqlx::SqlitePool::connect(":memory:").await.unwrap();

    sqlx::query("PRAGMA foreign_keys = ON;").execute(&pool).await.unwrap();

    sqlx::query(
        "CREATE TABLE requests (
            id TEXT PRIMARY KEY,
            webhook_id TEXT,
            method TEXT,
            headers TEXT,
            body TEXT,
            query TEXT,
            created_at TEXT
        )"
    )
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
        "CREATE TABLE webhook_configs (
            webhook_id TEXT PRIMARY KEY,
            status_code INTEGER,
            response_body TEXT,
            content_type TEXT,
            forward_url TEXT
        )"
    )
        .execute(&pool)
        .await
        .unwrap();

    let (tx, _) = broadcast::channel(10);

    Arc::new(AppState {
        db: Arc::new(db::Database { pool }),
        tx,
    })
}
