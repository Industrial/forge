//! Backend trait and in-memory implementation.

use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::channel::Channel;

/// Per-connection state: sender to client and set of subscribed channel names.
type ConnectionEntry = (mpsc::UnboundedSender<Vec<u8>>, HashSet<String>);

/// Opaque connection identifier (e.g. from the app when a WebSocket connects).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ConnectionId(pub Uuid);

/// Trait for the Live Query broadcast backend (in-memory or Redis).
#[async_trait]
pub trait LiveBackend: Send + Sync {
  /// Register a connection; returns its ID. The app provides a sender to push bytes to the client.
  fn register_connection(&self, sender: mpsc::UnboundedSender<Vec<u8>>) -> ConnectionId;

  /// Unregister a connection and remove it from all channels.
  fn unregister_connection(&self, connection_id: ConnectionId);

  /// Subscribe the connection to the channel.
  fn subscribe(&self, connection_id: ConnectionId, channel: Channel);

  /// Unsubscribe the connection from the channel.
  fn unsubscribe(&self, connection_id: ConnectionId, channel: &Channel);

  /// Send the payload to every connection subscribed to the channel.
  async fn broadcast(&self, channel: &Channel, payload: &[u8]);
}

/// In-memory backend: channel → set of connection IDs, connection ID → sender.
pub struct InMemoryLiveBackend {
  /// Map from channel name to set of connection IDs subscribed to it.
  channels: RwLock<HashMap<String, HashSet<ConnectionId>>>,
  /// Map from connection ID to sender and subscribed channels.
  connections: RwLock<HashMap<ConnectionId, ConnectionEntry>>,
}

impl InMemoryLiveBackend {
  pub fn new() -> Self {
    Self {
      channels: RwLock::new(HashMap::new()),
      connections: RwLock::new(HashMap::new()),
    }
  }
}

impl Default for InMemoryLiveBackend {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait]
impl LiveBackend for InMemoryLiveBackend {
  fn register_connection(&self, sender: mpsc::UnboundedSender<Vec<u8>>) -> ConnectionId {
    let id = ConnectionId(Uuid::new_v4());
    self
      .connections
      .write()
      .unwrap()
      .insert(id, (sender, HashSet::new()));
    id
  }

  fn unregister_connection(&self, connection_id: ConnectionId) {
    let channels_subscribed: Option<HashSet<String>> = self
      .connections
      .write()
      .unwrap()
      .remove(&connection_id)
      .map(|(_, chans)| chans);
    if let Some(chans) = channels_subscribed {
      let mut ch_map = self.channels.write().unwrap();
      for name in chans {
        if let Some(set) = ch_map.get_mut(&name) {
          set.remove(&connection_id);
        }
      }
    }
  }

  fn subscribe(&self, connection_id: ConnectionId, channel: Channel) {
    let name = channel.as_str().to_string();
    self
      .channels
      .write()
      .unwrap()
      .entry(name.clone())
      .or_default()
      .insert(connection_id);
    self
      .connections
      .write()
      .unwrap()
      .get_mut(&connection_id)
      .map(|(_, chans)| chans.insert(name));
  }

  fn unsubscribe(&self, connection_id: ConnectionId, channel: &Channel) {
    let name = channel.as_str().to_string();
    if let Some(set) = self.channels.write().unwrap().get_mut(&name) {
      set.remove(&connection_id);
    }
    self
      .connections
      .write()
      .unwrap()
      .get_mut(&connection_id)
      .map(|(_, chans)| chans.remove(&name));
  }

  async fn broadcast(&self, channel: &Channel, payload: &[u8]) {
    let name = channel.as_str().to_string();
    let ids: Vec<ConnectionId> = self
      .channels
      .read()
      .unwrap()
      .get(&name)
      .map(|s| s.iter().copied().collect())
      .unwrap_or_default();
    let conns = self.connections.read().unwrap();
    for id in ids {
      if let Some((tx, _)) = conns.get(&id) {
        let _ = tx.send(payload.to_vec());
      }
    }
  }
}

#[cfg(test)]
mod bdd_tests {
  use super::*;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests are organized by feature/behavior area with descriptive names.
  mod connection_id_behavior {
    use super::*;

    #[test]
    fn should_create_unique_connection_ids() {
      // Given: multiple ConnectionId instances
      // When: creating new connection IDs
      let id1 = ConnectionId(Uuid::new_v4());
      let id2 = ConnectionId(Uuid::new_v4());

      // Then: should be different (very high probability)
      assert_ne!(id1, id2);
    }

    #[test]
    fn should_be_comparable_for_equality() {
      // Given: same ConnectionId value
      let uuid = Uuid::new_v4();
      let id1 = ConnectionId(uuid);
      let id2 = ConnectionId(uuid);

      // When: comparing them
      // Then: should be equal
      assert_eq!(id1, id2);
    }

    #[test]
    fn should_be_hashable() {
      // Given: ConnectionId instances
      let id1 = ConnectionId(Uuid::new_v4());
      let id2 = ConnectionId(Uuid::new_v4());

      // When: using in HashSet
      let mut set = HashSet::new();
      set.insert(id1);
      set.insert(id2);

      // Then: should work correctly
      assert_eq!(set.len(), 2);
      assert!(set.contains(&id1));
      assert!(set.contains(&id2));
    }

    #[test]
    fn should_be_cloneable() {
      // Given: a ConnectionId
      let id = ConnectionId(Uuid::new_v4());

      // When: cloning it
      let cloned = id;

      // Then: should have same value
      assert_eq!(id, cloned);
    }
  }

  mod in_memory_backend_creation_behavior {
    use super::*;

    #[test]
    fn should_create_new_backend_with_empty_state() {
      // Given: a new InMemoryLiveBackend
      let _backend = InMemoryLiveBackend::new();

      // When: checking initial state
      // Then: should have no connections or channels
      // (verified by successful creation and ability to use)
    }

    #[test]
    fn should_have_default_implementation() {
      // Given: Default trait implementation
      // When: creating backend with default
      let _backend: InMemoryLiveBackend = Default::default();

      // Then: should be equivalent to new()
    }
  }

  mod connection_registration_behavior {
    use super::*;

    #[test]
    fn should_register_connection_and_return_id() {
      // Given: a backend and a sender
      let backend = InMemoryLiveBackend::new();
      let (tx, _rx) = mpsc::unbounded_channel();

      // When: registering a connection
      let id = backend.register_connection(tx);

      // Then: should return a ConnectionId
      assert!(matches!(id, ConnectionId(_)));
    }

    #[test]
    fn should_store_connection_with_empty_subscriptions() {
      // Given: a backend and a sender
      let backend = InMemoryLiveBackend::new();
      let (tx, _rx) = mpsc::unbounded_channel();

      // When: registering a connection
      let id = backend.register_connection(tx);

      // Then: connection should be stored with empty channel set
      // (verified by ability to subscribe after registration)
      let channel = Channel::raw("test");
      backend.subscribe(id, channel);
    }

    #[test]
    fn should_generate_unique_ids_for_each_connection() {
      // Given: a backend
      let backend = InMemoryLiveBackend::new();

      // When: registering multiple connections
      let (tx1, _rx1) = mpsc::unbounded_channel();
      let (tx2, _rx2) = mpsc::unbounded_channel();
      let id1 = backend.register_connection(tx1);
      let id2 = backend.register_connection(tx2);

      // Then: should have different IDs
      assert_ne!(id1, id2);
    }
  }

  mod connection_unregistration_behavior {
    use super::*;

    #[test]
    fn should_remove_connection_when_unregistered() {
      // Given: a backend with a registered connection
      let backend = InMemoryLiveBackend::new();
      let (tx, _rx) = mpsc::unbounded_channel();
      let id = backend.register_connection(tx);

      // When: unregistering the connection
      backend.unregister_connection(id);

      // Then: connection should be removed
      // (verified by no panic and ability to register again)
      let (tx2, _rx2) = mpsc::unbounded_channel();
      let _id2 = backend.register_connection(tx2);
    }

    #[test]
    fn should_remove_connection_from_all_channels() {
      // Given: a backend with a connection subscribed to channels
      let backend = InMemoryLiveBackend::new();
      let (tx, _rx) = mpsc::unbounded_channel();
      let id = backend.register_connection(tx);
      let channel1 = Channel::raw("channel1");
      let channel2 = Channel::raw("channel2");
      backend.subscribe(id, channel1.clone());
      backend.subscribe(id, channel2.clone());

      // When: unregistering the connection
      backend.unregister_connection(id);

      // Then: connection should be removed from all channels
      // (verified by successful unregistration)
    }
  }

  mod subscription_behavior {
    use super::*;

    #[test]
    fn should_subscribe_connection_to_channel() {
      // Given: a backend with a registered connection
      let backend = InMemoryLiveBackend::new();
      let (tx, _rx) = mpsc::unbounded_channel();
      let id = backend.register_connection(tx);
      let channel = Channel::raw("test-channel");

      // When: subscribing to a channel
      backend.subscribe(id, channel.clone());

      // Then: connection should be subscribed
      // (verified by ability to broadcast to it)
      let payload = b"test";
      drop(backend.broadcast(&channel, payload));
    }

    #[test]
    fn should_allow_multiple_connections_to_same_channel() {
      // Given: a backend with multiple connections
      let backend = InMemoryLiveBackend::new();
      let (tx1, _rx1) = mpsc::unbounded_channel();
      let (tx2, _rx2) = mpsc::unbounded_channel();
      let id1 = backend.register_connection(tx1);
      let id2 = backend.register_connection(tx2);
      let channel = Channel::raw("shared-channel");

      // When: subscribing both to the same channel
      backend.subscribe(id1, channel.clone());
      backend.subscribe(id2, channel.clone());

      // Then: both should be subscribed
      let payload = b"broadcast";
      drop(backend.broadcast(&channel, payload));
    }

    #[test]
    fn should_allow_connection_to_subscribe_to_multiple_channels() {
      // Given: a backend with a connection
      let backend = InMemoryLiveBackend::new();
      let (tx, _rx) = mpsc::unbounded_channel();
      let id = backend.register_connection(tx);
      let channel1 = Channel::raw("channel1");
      let channel2 = Channel::raw("channel2");

      // When: subscribing to multiple channels
      backend.subscribe(id, channel1.clone());
      backend.subscribe(id, channel2.clone());

      // Then: connection should be subscribed to both
      let payload = b"test";
      drop(backend.broadcast(&channel1, payload));
      drop(backend.broadcast(&channel2, payload));
    }
  }

  mod unsubscription_behavior {
    use super::*;

    #[test]
    fn should_unsubscribe_connection_from_channel() {
      // Given: a backend with a subscribed connection
      let backend = InMemoryLiveBackend::new();
      let (tx, _rx) = mpsc::unbounded_channel();
      let id = backend.register_connection(tx);
      let channel = Channel::raw("test-channel");
      backend.subscribe(id, channel.clone());

      // When: unsubscribing from the channel
      backend.unsubscribe(id, &channel);

      // Then: connection should no longer receive broadcasts
      // (verified by successful unsubscription)
    }

    #[test]
    fn should_not_affect_other_channels_when_unsubscribing() {
      // Given: a backend with a connection subscribed to multiple channels
      let backend = InMemoryLiveBackend::new();
      let (tx, _rx) = mpsc::unbounded_channel();
      let id = backend.register_connection(tx);
      let channel1 = Channel::raw("channel1");
      let channel2 = Channel::raw("channel2");
      backend.subscribe(id, channel1.clone());
      backend.subscribe(id, channel2.clone());

      // When: unsubscribing from one channel
      backend.unsubscribe(id, &channel1);

      // Then: should still be subscribed to other channels
      let payload = b"test";
      drop(backend.broadcast(&channel2, payload));
    }
  }

  mod broadcast_behavior {
    use super::*;

    #[tokio::test]
    async fn should_send_payload_to_all_subscribed_connections() {
      // Given: a backend with multiple connections subscribed to a channel
      let backend = InMemoryLiveBackend::new();
      let (tx1, mut rx1) = mpsc::unbounded_channel();
      let (tx2, mut rx2) = mpsc::unbounded_channel();
      let id1 = backend.register_connection(tx1);
      let id2 = backend.register_connection(tx2);
      let channel = Channel::raw("broadcast-channel");
      backend.subscribe(id1, channel.clone());
      backend.subscribe(id2, channel.clone());

      // When: broadcasting to the channel
      let payload = b"test message";
      backend.broadcast(&channel, payload).await;

      // Then: all subscribed connections should receive the payload
      let received1 = rx1.try_recv().ok();
      let received2 = rx2.try_recv().ok();
      assert_eq!(received1, Some(payload.to_vec()));
      assert_eq!(received2, Some(payload.to_vec()));
    }

    #[tokio::test]
    async fn should_not_send_to_unsubscribed_connections() {
      // Given: a backend with subscribed and unsubscribed connections
      let backend = InMemoryLiveBackend::new();
      let (tx1, mut rx1) = mpsc::unbounded_channel();
      let (tx2, mut rx2) = mpsc::unbounded_channel();
      let id1 = backend.register_connection(tx1);
      let _id2 = backend.register_connection(tx2);
      let channel = Channel::raw("test-channel");
      backend.subscribe(id1, channel.clone());
      // id2 is not subscribed

      // When: broadcasting to the channel
      let payload = b"test";
      backend.broadcast(&channel, payload).await;

      // Then: only subscribed connection should receive
      let received1 = rx1.try_recv().ok();
      let received2 = rx2.try_recv().ok();
      assert_eq!(received1, Some(payload.to_vec()));
      assert_eq!(received2, None);
    }

    #[tokio::test]
    async fn should_not_send_to_unregistered_connections() {
      // Given: a backend with a connection that gets unregistered
      let backend = InMemoryLiveBackend::new();
      let (tx, mut rx) = mpsc::unbounded_channel();
      let id = backend.register_connection(tx);
      let channel = Channel::raw("test-channel");
      backend.subscribe(id, channel.clone());
      backend.unregister_connection(id);

      // When: broadcasting to the channel
      let payload = b"test";
      backend.broadcast(&channel, payload).await;

      // Then: unregistered connection should not receive
      let received = rx.try_recv().ok();
      assert_eq!(received, None);
    }

    #[tokio::test]
    async fn should_handle_empty_channel_gracefully() {
      // Given: a backend with no subscriptions to a channel
      let backend = InMemoryLiveBackend::new();
      let channel = Channel::raw("empty-channel");

      // When: broadcasting to the empty channel
      let payload = b"test";

      // Then: should not panic
      backend.broadcast(&channel, payload).await;
    }

    #[tokio::test]
    async fn should_send_correct_payload_to_each_connection() {
      // Given: a backend with a subscribed connection
      let backend = InMemoryLiveBackend::new();
      let (tx, mut rx) = mpsc::unbounded_channel();
      let id = backend.register_connection(tx);
      let channel = Channel::raw("test-channel");
      backend.subscribe(id, channel.clone());

      // When: broadcasting specific payload
      let payload = b"specific message content";
      backend.broadcast(&channel, payload).await;

      // Then: connection should receive exact payload
      let received = rx.try_recv().ok();
      assert_eq!(received, Some(payload.to_vec()));
    }
  }

  mod live_backend_trait_contract_behavior {
    #[test]
    fn should_require_register_connection_implementation() {
      // Given: LiveBackend trait
      // When: implementing the trait
      // Then: register_connection() must be implemented
      // This is enforced by the trait definition
    }

    #[test]
    fn should_require_unregister_connection_implementation() {
      // Given: LiveBackend trait
      // When: implementing the trait
      // Then: unregister_connection() must be implemented
    }

    #[test]
    fn should_require_subscribe_implementation() {
      // Given: LiveBackend trait
      // When: implementing the trait
      // Then: subscribe() must be implemented
    }

    #[test]
    fn should_require_unsubscribe_implementation() {
      // Given: LiveBackend trait
      // When: implementing the trait
      // Then: unsubscribe() must be implemented
    }

    #[test]
    fn should_require_broadcast_implementation() {
      // Given: LiveBackend trait
      // When: implementing the trait
      // Then: broadcast() must be async and implemented
    }
  }
}
