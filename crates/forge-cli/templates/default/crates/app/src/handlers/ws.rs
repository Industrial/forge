//! WebSocket handler: forge-live connection and subscriptions. Clients send
//! `{"type":"subscribe","channel":"tasks"}` or `"audit-log"` or `"org:<uuid>"` for live updates.

use axum::{
  extract::ws::{Message, WebSocket, WebSocketUpgrade},
  extract::Extension,
  response::{IntoResponse, Response},
};
use forge::live::{Channel, InMemoryLiveBackend, LiveBackend};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::tasks::TaskState;

pub async fn handler(
  ws: WebSocketUpgrade,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Extension(task_state): Extension<Arc<TaskState>>,
) -> Response {
  tracing::debug!(target: "app::handlers", "route: GET /ws (upgrade)");
  let Some(backend) = live_backend else {
    return (axum::http::StatusCode::SERVICE_UNAVAILABLE, "Live Query not enabled").into_response();
  };
  let backend = backend.clone();
  let task_state = task_state.clone();
  ws.on_upgrade(move |socket| handle_socket(socket, backend, task_state))
}

async fn handle_socket(
  socket: WebSocket,
  live_backend: Arc<InMemoryLiveBackend>,
  task_state: Arc<TaskState>,
) {
  let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
  let main_tx = tx.clone();
  let conn_id = live_backend.register_connection(tx);

  let (mut socket_tx, mut socket_rx) = socket.split();
  let forward = tokio::spawn(async move {
    while let Some(payload) = rx.recv().await {
      let text = String::from_utf8_lossy(&payload).into_owned();
      if socket_tx.send(Message::Text(text.into())).await.is_err() {
        break;
      }
    }
  });

  while let Some(msg) = socket_rx.next().await {
    let msg = match msg {
      Ok(m) => m,
      Err(_) => break,
    };
    match msg {
      Message::Text(t) => {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&t) {
          if v.get("type").and_then(|x| x.as_str()) == Some("subscribe") {
            if let Some(ch) = v.get("channel").and_then(|x| x.as_str()) {
              if ch == "tasks" {
                live_backend.subscribe(conn_id, Channel::raw("tasks"));
                let tasks = task_state.store.read().await.clone();
                let payload =
                  serde_json::to_string(&serde_json::json!({ "type": "tasks", "tasks": tasks }))
                    .unwrap_or_default();
                let _ = main_tx.send(payload.into_bytes());
                continue;
              }
              if ch == "audit-log" {
                live_backend.subscribe(conn_id, Channel::raw("audit-log"));
                continue;
              }
              if let Some(org_part) = ch.strip_prefix("org:") {
                if let Ok(uuid) = uuid::Uuid::parse_str(org_part) {
                  live_backend.subscribe(conn_id, Channel::org(uuid));
                  continue;
                }
              }
            }
          }
        }
        tracing::debug!(target: "app::handlers::ws", "received: {}", t);
        let _ = main_tx.send(t.as_bytes().to_vec());
      }
      Message::Binary(b) => {
        tracing::debug!(target: "app::handlers::ws", "received {} bytes", b.len());
        let _ = main_tx.send(b.to_vec());
      }
      Message::Close(_) => break,
      _ => {}
    }
  }

  live_backend.unregister_connection(conn_id);
  forward.abort();
}
