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
  channels: RwLock<HashMap<String, HashSet<ConnectionId>>>,
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
