//! In-memory subscription store for invalidation events.
//! Subscribe returns server-assigned id; unsubscribe by id.
//! Long-lived stream for invalidation events; change worker matches events to subscriptions.

use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

/// Change event: published after CUD in generic handler; matching uses this to find affected subscriptions.
#[derive(Debug, Clone)]
pub struct ChangeEvent {
  pub model_id: String,
  pub resource_id: Uuid,
  pub action: String,
  pub organization_id: Option<Uuid>,
}

/// Invalidation hint: one message per affected subscription. Client refetches.
#[derive(Debug, Clone, serde::Serialize)]
pub struct InvalidationEvent {
  pub subscription_id: Uuid,
}

/// Minimal subscription record: entity + scope snapshot. Query spec (filter/sort/pagination) in params for matching.
#[derive(Debug, Clone)]
pub struct SubscriptionMeta {
  pub entity_id: String,
  pub organization_id: Uuid,
  pub role_id: Uuid,
  pub params: Option<serde_json::Value>,
}

/// In-memory store: subscription_id -> SubscriptionMeta; broadcast channels for change and invalidation events.
#[derive(Clone)]
pub struct SubscriptionStore {
  inner: std::sync::Arc<Mutex<HashMap<Uuid, SubscriptionMeta>>>,
  #[allow(dead_code)]
  invalidation_tx: tokio::sync::broadcast::Sender<InvalidationEvent>,
  change_tx: tokio::sync::broadcast::Sender<ChangeEvent>,
}

impl Default for SubscriptionStore {
  fn default() -> Self {
    Self::new()
  }
}

impl SubscriptionStore {
  pub fn new() -> Self {
    let (invalidation_tx, _) = tokio::sync::broadcast::channel(256);
    let (change_tx, _) = tokio::sync::broadcast::channel(256);
    Self {
      inner: std::sync::Arc::new(Mutex::new(HashMap::new())),
      invalidation_tx,
      change_tx,
    }
  }

  /// Publish a change event. Fire-and-forget; used by generic handler after create/update/delete.
  pub fn publish_change(&self, event: ChangeEvent) {
    let _ = self.change_tx.send(event);
  }

  /// Subscribe to change events (for matching worker).
  #[allow(dead_code)]
  pub fn subscribe_changes(&self) -> tokio::sync::broadcast::Receiver<ChangeEvent> {
    self.change_tx.subscribe()
  }

  /// Subscribe to invalidation events (for long-lived stream handler).
  pub fn subscribe_invalidations(&self) -> tokio::sync::broadcast::Receiver<InvalidationEvent> {
    self.invalidation_tx.subscribe()
  }

  /// Send an invalidation hint. One message per affected subscription.
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

  /// Get subscription by id (for matching/delivery).
  #[allow(dead_code)]
  pub fn get(&self, id: Uuid) -> Option<SubscriptionMeta> {
    self.inner.lock().unwrap().get(&id).cloned()
  }

  /// Scope-aware matching: return subscription ids affected by a change event.
  /// Matches when subscription entity_id == event.model_id and scope matches: platform-level
  /// (event.organization_id is None) matches all subs for that model; org-scoped event matches
  /// only subscriptions in that org.
  pub fn subscriptions_affected_by(&self, event: &ChangeEvent) -> Vec<Uuid> {
    let guard = self.inner.lock().unwrap();
    guard
      .iter()
      .filter(|(_id, meta)| {
        meta.entity_id == event.model_id
          && (event.organization_id.is_none()
            || event.organization_id == Some(meta.organization_id))
      })
      .map(|(id, _)| *id)
      .collect()
  }

  /// After matching: send an invalidation event for each affected subscription.
  /// Non-blocking: only in-process broadcast send; write path does not wait.
  pub fn notify_affected_by(&self, event: &ChangeEvent) {
    let ids = self.subscriptions_affected_by(event);
    for subscription_id in ids {
      self.send_invalidation(InvalidationEvent { subscription_id });
    }
  }

  /// Spawn a background task that receives change events and runs matching + delivery.
  /// Write path only calls publish_change and returns; this worker does not block it.
  pub fn spawn_change_worker(&self) {
    let store = self.clone();
    tokio::spawn(async move {
      let mut rx = store.subscribe_changes();
      while let Ok(event) = rx.recv().await {
        store.notify_affected_by(&event);
      }
    });
  }
}
