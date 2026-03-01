//! E2E test for Forge WebSocket support (014): prebuilt app exposes /ws echo, connect and round-trip.
//! Run via `bin/test-e2e`.

use std::time::Duration;

use forge_e2e_lib::cli;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

/// Single E2E test: prebuilt layout and WebSocket /ws echo.
/// Connects to ws://base/ws, sends "ping", expects "ping" back, then closes.
#[tokio::test]
async fn e2e_prebuilt_websocket_echo() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);

  let base = cli::e2e_base_url().expect("run e2e via bin/test-e2e (E2E_BASE_URL not set)");
  let ws_url = base
    .replace("http://", "ws://")
    .replace("https://", "wss://");
  let url = format!("{}/ws", ws_url);

  let (mut ws_stream, _) = tokio_tungstenite::connect_async(&url)
    .await
    .expect("WebSocket connect to /ws");

  ws_stream
    .send(Message::Text("ping".into()))
    .await
    .expect("send ping");

  let msg = tokio::time::timeout(Duration::from_secs(2), ws_stream.next())
    .await
    .expect("timeout waiting for reply")
    .expect("stream next")
    .expect("recv");
  match msg {
    Message::Text(t) => assert_eq!(t, "ping", "echo should return ping, got {:?}", t),
    other => panic!("expected Text(ping), got {:?}", other),
  }

  ws_stream.close(None).await.ok();
}
