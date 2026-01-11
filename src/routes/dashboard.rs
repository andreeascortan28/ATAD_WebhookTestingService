use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use std::sync::Arc;
use crate::{AppState, models::StoredRequest};
use serde_json;

/// Helper to fetch all requests for a given webhook.
impl AppState {
    pub async fn get_requests(&self, webhook_id: &str) -> Vec<StoredRequest> {
        sqlx::query_as::<_, StoredRequest>(
            "SELECT * FROM requests WHERE webhook_id = ?1 ORDER BY created_at DESC",
        )
            .bind(webhook_id)
            .fetch_all(&self.db.pool)
            .await
            .unwrap_or_default()
    }
}

/// Dashboard handler
pub async fn dashboard_handler(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Fetch stored requests
    let requests = state.get_requests(&id).await;
    let requests_json = serde_json::to_string(&requests).unwrap_or_else(|_| "[]".to_string());

    // Render HTML
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Webhook Dashboard</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-gray-100 min-h-screen text-gray-800">
<div class="max-w-6xl mx-auto px-4 py-6">
    <h1 class="text-3xl font-bold mb-1">Webhook Dashboard</h1>
    <p class="text-sm text-gray-500 mb-4">
        Webhook ID: <code class="bg-gray-200 px-2 py-1 rounded">{id}</code>
    </p>

    <div class="flex items-center justify-between mb-4">
        <p class="text-sm">
            Total requests: <span id="count" class="font-semibold">0</span>
        </p>
        <span id="ws-status" class="text-xs px-2 py-1 rounded bg-gray-300 text-gray-700">
            Connecting…
        </span>
    </div>

    <div id="requests" class="flex flex-col gap-4"></div>
</div>

<script>
const WEBHOOK_ID = "{id}";
const INITIAL_REQUESTS = {requests_json};
const container = document.getElementById("requests");
const countEl = document.getElementById("count");
const statusEl = document.getElementById("ws-status");

function updateCount() {{
    countEl.textContent = container.children.length;
}}

function replayRequest(id) {{
    fetch(`/replay/${{id}}`, {{ method: 'POST', headers: {{ 'Content-Type': 'application/json' }}, body: JSON.stringify({{ target: "" }}) }})
        .then(res => {{
            if(res.ok) console.log("Request replayed:", id);
            else console.error("Failed to replay request:", id);
        }})
        .catch(err => console.error("Replay error:", err));
}}

function renderRequest(req, highlight=false) {{
    const el = document.createElement("div");
    el.className = "bg-white border rounded shadow-sm p-4 text-sm font-mono";

    if (highlight) {{
        el.classList.add("ring", "ring-blue-400");
        setTimeout(() => el.classList.remove("ring", "ring-blue-400"), 1200);
    }}

    el.innerHTML = `
        <div class="flex justify-between items-center mb-2">
            <span class="text-xs text-gray-500 break-all">ID: \${{req.id}}</span>
            <button
                class="text-xs text-blue-600 underline"
                onclick="replayRequest('\${{req.id}}')">
                Replay
            </button>
        </div>
        <div class="text-xs text-gray-400 mb-2">\${{new Date(req.created_at).toLocaleString()}}</div>

        <details class="mb-1">
            <summary class="cursor-pointer font-semibold text-gray-700">Headers</summary>
            <pre class="bg-gray-100 p-2 mt-1 rounded"></pre>
        </details>

        <details class="mb-1">
            <summary class="cursor-pointer font-semibold text-gray-700">Query</summary>
            <pre class="bg-gray-100 p-2 mt-1 rounded"></pre>
        </details>

        <details open>
            <summary class="cursor-pointer font-semibold text-gray-700">Body</summary>
            <pre class="bg-gray-100 p-2 mt-1 rounded"></pre>
        </details>
    `;

    // XSS-safe rendering
    const pres = el.querySelectorAll("pre");
    pres[0].textContent = req.headers || "";
    pres[1].textContent = req.query || "";
    pres[2].textContent = req.body || "";

    container.prepend(el);
    updateCount();
}};

// Render initial requests
INITIAL_REQUESTS.forEach(req => renderRequest(req));
updateCount();

// WebSocket connection for live updates
const wsUrl = location.origin.replace(/^http/, "ws") + `/ws/${{WEBHOOK_ID}}`;
const ws = new WebSocket(wsUrl);

ws.onopen = () => {{
    statusEl.textContent = "Connected";
    statusEl.className = "text-xs px-2 py-1 rounded bg-green-500 text-white";
}};

ws.onmessage = (event) => {{
    try {{
        const data = JSON.parse(event.data);
        if (!data || !data.id) return;
        renderRequest(data, true); // highlight new request
    }} catch (err) {{
        console.error("WS parse error:", err);
    }}
}};

ws.onclose = () => {{
    statusEl.textContent = "Disconnected — reconnecting…";
    statusEl.className = "text-xs px-2 py-1 rounded bg-red-500 text-white";
    setTimeout(() => location.reload(), 3000);
}};
</script>
</body>
</html>
"#,
        id = id,
        requests_json = requests_json
    );

    Html(html)
}
