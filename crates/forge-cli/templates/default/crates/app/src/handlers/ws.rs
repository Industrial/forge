//! WebSocket handler: forge-live connection and subscriptions. Authenticated
//! connections receive server-derived channel subscriptions (see docs/021).
//! Scope (org/role) comes from X-Organization-Id and X-Role-Name headers.

use axum::{
  extract::ws::{Message, WebSocket, WebSocketUpgrade},
  extract::{Extension, Request, State},
  http::StatusCode,
  response::{IntoResponse, Response},
};
use forge_db::DbConnection;
use forge_live::{Channel, InMemoryLiveBackend, LiveBackend};
use forge_auth::token_auth::RequireAuth;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::handlers::auth::{channels_from_permissions, get_scope_from_headers_map, resolve_permissions};
use crate::tasks::TaskState;
use db::auth::Backend;

pub async fn handler(
  ws: WebSocketUpgrade,
  Request(req): Request,
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Extension(task_state): Extension<Arc<TaskState>>,
) -> Response {
  tracing::debug!(target: "app::handlers", "route: GET /ws (upgrade)");
  let Some(backend) = live_backend else {
    return (StatusCode::SERVICE_UNAVAILABLE, "Live Query not enabled").into_response();
  };
  let profile = get_scope_from_headers_map(req.headers(), &user, &db).await;
  let permissions = resolve_permissions(&db, &user, profile.as_ref()).await;
  let current_org_id = profile.as_ref().map(|p| p.org_id);
  let channels = channels_from_permissions(&permissions, current_org_id);
  let backend = backend.clone();
  let task_state = task_state.clone();
  ws.on_upgrade(move |socket| handle_socket(socket, backend, task_state, channels))
}

async fn handle_socket(
  socket: WebSocket,
  live_backend: Arc<InMemoryLiveBackend>,
  task_state: Arc<TaskState>,
  channels: Vec<Channel>,
) {
  let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
  let main_tx = tx.clone();
  let conn_id = live_backend.register_connection(tx);

  for ch in &channels {
    live_backend.subscribe(conn_id, ch.clone());
  }
  if channels.iter().any(|c| c.as_str() == "tasks") {
    let tasks = task_state.store.read().await.clone();
    let payload =
      serde_json::to_string(&serde_json::json!({ "type": "tasks", "tasks": tasks }))
        .unwrap_or_default();
    let _ = main_tx.send(payload.into_bytes());
  }

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
      Message::Close(_) => break,
      _ => {}
    }
  }

  live_backend.unregister_connection(conn_id);
  forward.abort();
}
