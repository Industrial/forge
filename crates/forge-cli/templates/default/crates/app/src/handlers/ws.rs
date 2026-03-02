//! WebSocket handler: echo demo, tasks channel, and audit-log channel. Clients send `{"type":"subscribe","channel":"tasks"}` or `{"type":"subscribe","channel":"audit-log"}` for live updates.

use axum::{
  extract::ws::{Message, WebSocket, WebSocketUpgrade},
  extract::Extension,
  response::Response,
};
use std::sync::Arc;

use crate::tasks::TaskState;

pub async fn handler(
  ws: WebSocketUpgrade,
  Extension(task_state): Extension<Arc<TaskState>>,
) -> Response {
  tracing::debug!(target: "app::handlers", "route: GET /ws (upgrade)");
  let task_state = task_state.clone();
  ws.on_upgrade(move |socket| handle_socket(socket, task_state))
}

async fn handle_socket(mut socket: WebSocket, task_state: Arc<TaskState>) {
  let mut tasks_rx = task_state.broadcast.subscribe();
  let mut audit_rx = task_state.audit_broadcast.subscribe();
  let store = task_state.store.clone();
  let mut subscribed_tasks = false;
  let mut subscribed_audit_log = false;

  loop {
    tokio::select! {
      Some(msg) = socket.recv() => {
        let msg = match msg {
          Ok(m) => m,
          Err(_) => return,
        };
        match &msg {
          Message::Text(t) => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(t) {
              if v.get("type").and_then(|x| x.as_str()) == Some("subscribe") {
                if v.get("channel").and_then(|x| x.as_str()) == Some("tasks") {
                  subscribed_tasks = true;
                  let tasks = store.read().await.clone();
                  let payload = match serde_json::to_string(&serde_json::json!({ "type": "tasks", "tasks": tasks })) {
                    Ok(s) => s,
                    Err(_) => continue,
                  };
                  if socket.send(Message::Text(payload.into())).await.is_err() {
                    return;
                  }
                  continue;
                }
                if v.get("channel").and_then(|x| x.as_str()) == Some("audit-log") {
                  subscribed_audit_log = true;
                  continue;
                }
              }
            }
            tracing::info!(target: "forge::ws", "received: {}", t);
            if socket.send(msg).await.is_err() {
              return;
            }
          }
          Message::Binary(b) => {
            tracing::info!(target: "forge::ws", "received: {} bytes, sending", b.len());
            if socket.send(msg).await.is_err() {
              return;
            }
          }
          _ => {}
        }
      }
      Ok(tasks) = tasks_rx.recv() => {
        if subscribed_tasks {
          let payload = match serde_json::to_string(&serde_json::json!({ "type": "tasks", "tasks": tasks })) {
            Ok(s) => s,
            Err(_) => continue,
          };
          if socket.send(Message::Text(payload.into())).await.is_err() {
            return;
          }
        }
      }
      Ok(entry) = audit_rx.recv() => {
        if subscribed_audit_log {
          let payload = match serde_json::to_string(&serde_json::json!({ "type": "audit_log", "entry": entry })) {
            Ok(s) => s,
            Err(_) => continue,
          };
          if socket.send(Message::Text(payload.into())).await.is_err() {
            return;
          }
        }
      }
      else => break,
    }
  }
}
