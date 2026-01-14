mod common;

use webhook_tester::db::Database;
use webhook_tester::models::{StoredRequest, WebhookConfig};
use sqlx::SqlitePool;

#[tokio::test]
async fn create_and_fetch_request() {
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    let db = Database { pool };

    sqlx::query("CREATE TABLE requests (
        id TEXT PRIMARY KEY,
        webhook_id TEXT,
        method TEXT,
        headers TEXT,
        body TEXT,
        query TEXT,
        created_at TEXT
    )").execute(&db.pool).await.unwrap();

    let req = StoredRequest {
        id: "1".into(),
        webhook_id: "wh".into(),
        method: "POST".into(),
        headers: "{}".into(),
        body: "body".into(),
        query: "{}".into(),
        created_at: "now".into(),
    };

    db.store_request(&req).await.unwrap();
    let fetched = db.get_request("1").await.unwrap();

    assert_eq!(fetched.body, "body");
}
