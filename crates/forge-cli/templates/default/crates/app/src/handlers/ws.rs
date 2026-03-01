use axum::{
  extract::ws::{Message, WebSocket, WebSocketUpgrade},
  response::Response,
};

pub async fn handler(ws: WebSocketUpgrade) -> Response {
  ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
  while let Some(msg) = socket.recv().await {
    let msg = match msg {
      Ok(m) => m,
      Err(_) => return,
    };
    match &msg {
      Message::Text(t) => {
        tracing::info!(target: "forge::ws", "received: {}", t);
        tracing::info!(target: "forge::ws", "sending: {}", t);
      }
      Message::Binary(b) => tracing::info!(target: "forge::ws", "received: {} bytes, sending", b.len()),
      _ => {}
    }
    if socket.send(msg).await.is_err() {
      return;
    }
  }
}
