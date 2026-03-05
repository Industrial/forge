//! In-memory subscription store (Epic 8). Subscribe returns server-assigned id; unsubscribe by id.
//! Long-lived stream for invalidation events (Epic 8). Epic 9 will replace store with cache layer.

use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

/// Invalidation hint: one message per affected subscription (Epic 8 §3). Client refetches.
#[derive(Debug, Clone, serde::Serialize)]
pub struct InvalidationEvent {
  pub subscription_id: Uuid,
}

/// Minimal subscription record: entity + scope snapshot. Query spec (filter/sort/pagination) in params for matching (Epic 9).
#[derive(Debug, Clone)]
pub struct SubscriptionMeta {
  pub entity_id: String,
  pub organization_id: uuid::Uuid,
  pub role_id: uuid::Uuid,
  pub params: Option<serde_json::Value>,
}

/// In-memory store: subscription_id -> SubscriptionMeta; broadcast channel for invalidation events.
#[derive(Clone)]
pub struct SubscriptionStore {
  inner: std::sync::Arc<Mutex<HashMap<Uuid, SubscriptionMeta>>>,
  #[allow(dead_code)]
  invalidation_tx: tokio::sync::broadcast::Sender<InvalidationEvent>,
}

impl Default for SubscriptionStore {
  fn default() -> Self {
    Self::new()
  }
}

impl SubscriptionStore {
  pub fn new() -> Self {
    let (invalidation_tx, _) = tokio::sync::broadcast::channel(256);
    Self {
      inner: std::sync::Arc::new(Mutex::new(HashMap::new())),
      invalidation_tx,
    }
  }

  /// Subscribe to invalidation events (for long-lived stream handler). Epic 9 will call send_invalidation.
  pub fn subscribe_invalidations(&self) -> tokio::sync::broadcast::Receiver<InvalidationEvent> {
    self.invalidation_tx.subscribe()
  }

  /// Send an invalidation hint (Epic 9: after matching). One message per affected subscription.
  #[allow(dead_code)]
  pub fn send_invalidation(&self, event: InvalidationEvent) {
    let _ = self.invalidation_tx.send(event);
  }

  /// Register a subscription; returns server-assigned id.
  pub fn subscribe(&self, meta: SubscriptionMeta) -> Uuid {
    let id = Uuid::new_v4();
    let _ = self.inner.lock().unwrap().insert(id, meta);
    id
  }

  /// Remove subscription by id. Returns true if it existed.
  pub fn unsubscribe(&self, id: Uuid) -> bool {
    self.inner.lock().unwrap().remove(&id).is_some()
  }

  /// Get subscription by id (for matching/delivery in Epic 9).
  #[allow(dead_code)]
  pub fn get(&self, id: Uuid) -> Option<SubscriptionMeta> {
    self.inner.lock().unwrap().get(&id).cloned()
  }
}
