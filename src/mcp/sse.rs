use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
    routing::{get, post},
    Json, Router,
};
use futures_util::stream::{Stream, StreamExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{info, warn};
use uuid::Uuid;

use crate::mcp::dispatcher::Dispatcher;
use crate::mcp::protocol::JsonRpcRequest;

pub struct SseServer {
    dispatcher: Arc<Dispatcher>,
    sessions: Arc<Mutex<HashMap<String, mpsc::Sender<Value>>>>,
}

impl SseServer {
    pub fn new(dispatcher: Arc<Dispatcher>) -> Self {
        Self {
            dispatcher,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn run(self: Arc<Self>, addr: SocketAddr) -> anyhow::Result<()> {
        let app = Router::new()
            .route("/sse", get(sse_handler))
            .route("/message/{session_id}", post(message_handler))
            .with_state(self);

        info!("MCP SSE Server listening on http://{}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn sse_handler(
    State(server): State<Arc<SseServer>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let session_id = Uuid::new_v4().to_string();
    let (tx, rx) = mpsc::channel(100);

    {
        let mut sessions = server.sessions.lock().await;
        sessions.insert(session_id.clone(), tx);
    }

    info!("New SSE session started: {}", session_id);

    let endpoint_url = format!("/message/{}", session_id);
    let initial_event = Event::default()
        .event("endpoint")
        .data(endpoint_url);

    let stream = ReceiverStream::new(rx).map(|msg| {
        Ok(Event::default()
            .event("message")
            .data(serde_json::to_string(&msg).unwrap_or_default()))
    });

    let combined_stream = futures_util::stream::once(async move { Ok(initial_event) })
        .chain(stream);

    Sse::new(combined_stream)
}

async fn message_handler(
    State(server): State<Arc<SseServer>>,
    Path(session_id): Path<String>,
    Json(payload): Json<JsonRpcRequest>,
) -> Json<Value> {
    if let Some(response) = server.dispatcher.dispatch(payload).await {
        let mut sessions = server.sessions.lock().await;
        if let Some(tx) = sessions.get(&session_id) {
            if let Err(e) = tx.send(serde_json::to_value(&response).unwrap()).await {
                 warn!("Failed to send response to SSE stream for session {}: {}", session_id, e);
                 sessions.remove(&session_id);
            }
        }
    }

    Json(json!({"status": "accepted"}))
}
