use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use std::sync::Arc;
use crate::{AppState, models::StoredRequest};
use serde_json;

impl AppState {
    /// Helper to fetch all requests for a given webhook.
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

pub async fn dashboard_handler(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Fetch existing requests for this webhook
    let requests = state.get_requests(&id).await;

    println!(
        "[DASHBOARD] Requested dashboard for webhook_id = {} ({} requests found)",
        id,
        requests.len()
    );

    for r in &requests {
        println!("    -> Found request ID: {} (method: {})", r.id, r.method);
    }

    let requests_json = serde_json::to_string(&requests).unwrap_or("[]".to_string());

    let html = format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Webhook Dashboard</title>
            <script src="https://cdn.tailwindcss.com"></script>
        </head>
        <body class="bg-gray-50 text-gray-800 min-h-screen">
            <div class="max-w-5xl mx-auto py-8 px-4">
                <h1 class="text-3xl font-bold mb-2">Webhook Dashboard</h1>
                <p class="text-sm text-gray-500 mb-6">Webhook ID: <code class="bg-gray-200 px-1 rounded">{id}</code></p>

                <div class="border-t border-gray-300 my-4"></div>
                <div id="requests" class="flex flex-col-reverse gap-4 overflow-y-auto max-h-[70vh]"></div>
            </div>

            <script>
                const WEBHOOK_ID = "{id}";
                const INITIAL_REQUESTS = {requests_json};
                console.log("Loaded INITIAL_REQUESTS:", INITIAL_REQUESTS);

                document.addEventListener("DOMContentLoaded", () => {{
                    const requestsDiv = document.getElementById("requests");

                    if (!requestsDiv) {{
                        console.error("Requests container not found!");
                        return;
                    }}

                    let initialRequests = INITIAL_REQUESTS;
                    if (typeof INITIAL_REQUESTS === "string") {{
                        try {{
                            initialRequests = JSON.parse(INITIAL_REQUESTS);
                        }} catch (err) {{
                            console.error("Error parsing INITIAL_REQUESTS:", err);
                            initialRequests = [];
                        }}
                    }}

                    function renderRequest(req) {{
                        const el = document.createElement("div");
                        el.className = "req border border-gray-300 bg-white shadow-sm rounded-lg p-4 text-sm font-mono whitespace-pre-wrap break-words";

                        el.innerHTML = `
                            <div class="flex justify-between items-center mb-2">
                                <span class="text-xs text-gray-500">ID: ${{req.id}}</span>
                                <span class="text-xs text-gray-400">${{new Date(req.created_at).toLocaleString()}}</span>
                            </div>
                            <div class="mb-2">
                                <span class="text-blue-600 font-semibold">${{req.method}}</span>
                                <span class="text-gray-600 ml-2">Webhook: ${{req.webhook_id}}</span>
                            </div>
                            <details class="mb-1">
                                <summary class="cursor-pointer text-gray-700 font-medium">Headers</summary>
                                <pre class="bg-gray-100 p-2 rounded mt-1">${{req.headers}}</pre>
                            </details>
                            <details class="mb-1">
                                <summary class="cursor-pointer text-gray-700 font-medium">Query</summary>
                                <pre class="bg-gray-100 p-2 rounded mt-1">${{req.query}}</pre>
                            </details>
                            <details open>
                                <summary class="cursor-pointer text-gray-700 font-medium">Body</summary>
                                <pre class="bg-gray-100 p-2 rounded mt-1">${{req.body}}</pre>
                            </details>
                        `;

                        requestsDiv.prepend(el);
                    }}

                    initialRequests.forEach(renderRequest);
                    console.log("Rendered initial requests:", initialRequests);

                    const wsUrl = `${{location.origin.replace(/^http/, "ws")}}/ws/${{WEBHOOK_ID}}`;
                    const ws = new WebSocket(wsUrl);

                    ws.onopen = () => {{
                        console.log("Connected to WebSocket");
                        showStatus("Connected", "green");
                    }};

                    ws.onmessage = (event) => {{
                        try {{
                            const data = JSON.parse(event.data);
                            if (data.status) return;
                            renderRequest(data);
                            showStatus("New request received", "blue");
                        }} catch (err) {{
                            console.error("Error parsing WS message:", err);
                        }}
                    }};

                    ws.onclose = () => {{
                        console.warn("WebSocket disconnected");
                        showStatus("Disconnected — reconnecting...", "red");
                        setTimeout(() => location.reload(), 3000);
                    }};

                    function showStatus(msg, color) {{
                        let bar = document.getElementById("status-bar");
                        if (!bar) {{
                            bar = document.createElement("div");
                            bar.id = "status-bar";
                            bar.className = "fixed top-2 right-2 px-3 py-1 rounded text-white text-sm shadow";
                            document.body.appendChild(bar);
                        }}
                        bar.textContent = msg;
                        bar.className = `fixed top-2 right-2 px-3 py-1 rounded text-white text-sm shadow bg-${{color}}-500`;
                        setTimeout(() => (bar.style.opacity = 0.8), 100);
                    }}
                }});
            </script>
        </body>
        </html>
        "#,
        id = id,
        requests_json = requests_json
    );

    Html(html)
}
