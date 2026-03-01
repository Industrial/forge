# Real-Time: WebSockets & Server-Sent Events

Forge supports real-time communication using **Axum’s built-in WebSocket and SSE support**. No extra crates are required; the framework enables the `ws` and `sse` features on the `axum` dependency.

## Objectives

1. **WebSockets:** Bidirectional, long-lived connections for chat, collaboration, or any low-latency client↔server messaging.
2. **Server-Sent Events (SSE):** One-way server→client streaming over HTTP for feeds, notifications, and live updates.
3. **Minimal surface:** Use only base Axum APIs; no Redis or third-party real-time crates in the core.

## When to use which

| Use case | Prefer |
|----------|--------|
| Bidirectional (chat, games, collaborative editing) | WebSockets |
| Server push only (notifications, live logs, dashboards) | SSE |
| Simple reconnection and Last-Event-ID semantics | SSE |

## WebSockets (Axum `ws` feature)

Axum provides `axum::extract::ws::{WebSocketUpgrade, WebSocket}`. The handler accepts a `WebSocketUpgrade`, performs the HTTP upgrade, and runs a callback with the `WebSocket` stream.

### Example

```rust
use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};

// Mount with .route("/ws", get(ws_handler)) or any(ws_handler) if you need CONNECT
async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(m) => m,
            Err(_) => return,
        };
        if socket.send(msg).await.is_err() {
            return;
        }
    }
}
```

### With app state

Use `State` to pass database or broadcast channels into the upgrade callback:

```rust
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}
```

### Configuration

`WebSocketUpgrade` supports `.read_buffer_size()`, `.write_buffer_size()`, `.max_message_size()`, `.max_frame_size()`, `.protocols()`, and `.on_failed_upgrade()` for logging or metrics.

## Server-Sent Events (Axum `sse` feature)

Axum provides `axum::response::sse::{Sse, Event, KeepAlive}`. The handler returns `Sse::new(stream)` where the stream yields `Result<Event, E>`. Use `.keep_alive(KeepAlive::default())` to keep the connection open.

### Example

```rust
use axum::{
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
    Router,
};
use futures_util::stream;
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt;

async fn sse_handler() -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::repeat_with(|| Event::default().data("ping"))
        .map(Ok)
        .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(KeepAlive::default())
}
```

### From a broadcast channel

To push events from in-process pub/sub (e.g. `tokio::sync::broadcast`), create a stream that receives from the channel and maps to `Event`:

```rust
// In handler: subscribe to a broadcast channel and return Sse::new(stream).keep_alive(...)
// The stream yields Event::default().data(json_or_text) for each received message.
```

## Integration with Forge

- **Routes:** Add WebSocket and SSE routes via `App::route()` or by merging a `Router` that uses `get(ws_handler)` / `get(sse_handler)`. Axum’s `ws` extractor typically uses `GET` (HTTP/1.1) or `CONNECT`; use `any(ws_handler)` if you need both.
- **Auth:** Run authentication (e.g. session or token) before calling `ws.on_upgrade(...)`. Reject the request with 401/403 if unauthenticated so the upgrade never occurs.
- **State:** Pass `DatabaseConnection` or shared channels via Axum’s `State` into the upgrade or SSE handler.

## Status

**Implemented:** Forge enables Axum’s `ws` feature. Application code can register WebSocket (and SSE, if the app adds the `sse` feature) handlers using the patterns above. No Forge-specific real-time API layer is provided; use Axum’s APIs directly.

- **Scaffold:** `forge new` generates an app with a `/ws` route and an echo handler (`handlers::ws::handler`) so new projects can use WebSockets out of the box. The app’s `axum` dependency includes the `ws` feature.
- **E2E:** Test `014_websockets` connects to `ws://base/ws`, sends a text message, and asserts the echo response.

**Not in scope (this document):** Redis Pub/Sub, multi-node scaling, or typed WebSocket message crates. Those can be added at the application layer.
