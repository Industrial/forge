//! Redis pub/sub backend for multi-instance Live Query.
//!
//! When using multiple server instances, each instance runs a RedisLiveBackend that
//! keeps local connection state and uses Redis to fan out broadcasts to all instances.

use async_trait::async_trait;
use futures_util::StreamExt;
use redis::AsyncCommands;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::backend::{ConnectionId, LiveBackend};
use crate::channel::Channel;

enum RedisCommand {
  Subscribe(String),
  Publish(String, Vec<u8>),
}

/// Live Query backend that uses Redis pub/sub so broadcasts reach all server instances.
///
/// Each instance keeps local connection state; `broadcast` publishes to Redis,
/// and a background task subscribes to Redis and fans out received messages to
/// local connections. Use this when running multiple app instances behind a load balancer.
pub struct RedisLiveBackend {
  channels: RwLock<HashMap<String, HashSet<ConnectionId>>>,
  connections: RwLock<HashMap<ConnectionId, (mpsc::UnboundedSender<Vec<u8>>, HashSet<String>)>>,
  command_tx: mpsc::UnboundedSender<RedisCommand>,
}

impl RedisLiveBackend {
  /// Build a Redis backend from a Redis URL (e.g. `redis://127.0.0.1/`).
  ///
  /// Spawns a background task that subscribes to channels and publishes;
  /// all instances sharing the same Redis will receive each broadcast.
  pub async fn connect(redis_url: &str) -> Result<Arc<Self>, redis::RedisError> {
    let client = redis::Client::open(redis_url)?;
    let mut pub_conn = client.get_multiplexed_async_connection().await?;
    let pubsub = client.get_async_pubsub().await?;

    let (command_tx, mut command_rx) = mpsc::unbounded_channel::<RedisCommand>();
    let channels: RwLock<HashMap<String, HashSet<ConnectionId>>> = RwLock::new(HashMap::new());
    let connections: RwLock<
      HashMap<ConnectionId, (mpsc::UnboundedSender<Vec<u8>>, HashSet<String>)>,
    > = RwLock::new(HashMap::new());

    let backend = Arc::new(Self {
      channels,
      connections,
      command_tx,
    });

    let backend_task = backend.clone();
    let (mut pubsub_sink, mut pubsub_stream) = pubsub.split();
    tokio::spawn(async move {
      let mut subscribed: HashSet<String> = HashSet::new();
      loop {
        tokio::select! {
          Some(cmd) = command_rx.recv() => match cmd {
            RedisCommand::Subscribe(ch) => {
              if subscribed.insert(ch.clone()) {
                if let Err(e) = pubsub_sink.subscribe(ch.clone()).await {
                  error!(channel = %ch, error = %e, "redis subscribe failed");
                  subscribed.remove(&ch);
                } else {
                  debug!(channel = %ch, "redis subscribed");
                }
              }
            }
            RedisCommand::Publish(ch, payload) => {
              if let Err(e) = pub_conn.publish::<_, _, ()>(ch.clone(), payload).await {
                error!(channel = %ch, error = %e, "redis publish failed");
              }
            }
          },
          msg = pubsub_stream.next() => match msg {
            Some(m) => {
              let channel = m.get_channel_name().to_string();
              let payload: Result<Vec<u8>, _> = m.get_payload();
              match payload {
                Ok(p) => RedisLiveBackend::fan_out_local(&backend_task, &channel, &p),
                Err(e) => error!(error = %e, "redis message get_payload failed"),
              }
            }
            None => break,
          }
        }
      }
    });

    info!(url = %redis_url, "Redis Live Query backend started");
    Ok(backend)
  }

  fn fan_out_local(backend: &Arc<Self>, channel: &str, payload: &[u8]) {
    let ids: Vec<ConnectionId> = {
      let ch = backend.channels.read().unwrap();
      ch.get(channel)
        .map(|s| s.iter().copied().collect())
        .unwrap_or_default()
    };
    let conns = backend.connections.read().unwrap();
    for id in ids {
      if let Some((tx, _)) = conns.get(&id) {
        let _ = tx.send(payload.to_vec());
      }
    }
  }
}

#[async_trait]
impl LiveBackend for RedisLiveBackend {
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
    let _ = self.command_tx.send(RedisCommand::Subscribe(name.clone()));
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
    let _ = self
      .command_tx
      .send(RedisCommand::Publish(name, payload.to_vec()));
  }
}
