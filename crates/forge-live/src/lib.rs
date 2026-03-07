//! Live Query: channel-based broadcast for real-time sync.
//!
//! Clients subscribe to channels (e.g. `org:{org_id}`). Handlers call `broadcast`
//! after mutations; all connections in that channel receive the event.
//! The app owns the transport (e.g. WebSocket); this crate provides the registry and broadcast.

mod backend;
mod channel;
mod event;
mod subscription;
#[cfg(feature = "redis")]
mod redis_backend;

pub use backend::{ConnectionId, InMemoryLiveBackend, LiveBackend};
pub use channel::Channel;
pub use event::LiveEvent;
pub use subscription::{
  ChangeEvent, InvalidationEvent, SubscriptionMeta, SubscriptionStore,
};
#[cfg(feature = "redis")]
pub use redis_backend::RedisLiveBackend;

use std::sync::Arc;
use uuid::Uuid;

/// Build a channel name for an organization. Use with [LiveBackend::broadcast].
pub fn org_channel(org_id: Uuid) -> Channel {
  Channel::org(org_id)
}

/// Broadcast an event to all connections subscribed to the given channel.
/// Serializes the event as JSON.
pub async fn broadcast_to_channel<B: LiveBackend + ?Sized>(
  backend: &Arc<B>,
  channel: &Channel,
  event: &LiveEvent,
) -> Result<(), forge_core::Error> {
  let payload = serde_json::to_vec(event).map_err(|e| forge_core::Error::Generic(e.to_string()))?;
  backend.broadcast(channel, &payload).await;
  Ok(())
}

/// Convenience: broadcast to `org:{org_id}`.
pub async fn broadcast_to_org<B: LiveBackend + ?Sized>(
  backend: &Arc<B>,
  org_id: Uuid,
  event: &LiveEvent,
) -> Result<(), forge_core::Error> {
  broadcast_to_channel(backend, &org_channel(org_id), event).await
}

/// No-op for in-memory backend; reserved for future DB/Redis-backed connection cleanup.
/// Called periodically by the app cron when Live Query is enabled.
pub async fn sweep_expired_connections<C>(_db: C) {
  // In-memory backend does not persist connections; Redis/DB backends could implement cleanup here.
}
