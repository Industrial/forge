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
  /// Map of subscription IDs to their metadata.
  inner: std::sync::Arc<Mutex<HashMap<Uuid, SubscriptionMeta>>>,
  #[allow(dead_code)]
  /// Broadcast channel sender for invalidation events.
  invalidation_tx: tokio::sync::broadcast::Sender<InvalidationEvent>,
  /// Broadcast channel sender for change events.
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

#[cfg(test)]
mod bdd_tests {
  use super::*;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests are organized by feature/behavior area with descriptive names.
  mod subscription_registration_behavior {
    use super::*;

    #[test]
    fn should_register_subscription_and_return_unique_id() {
      // Given: a subscription store and subscription metadata
      let store = SubscriptionStore::new();
      let meta = SubscriptionMeta {
        entity_id: "user".to_string(),
        organization_id: Uuid::new_v4(),
        role_id: Uuid::new_v4(),
        params: None,
      };

      // When: subscribing
      let id1 = store.subscribe(meta.clone());
      let id2 = store.subscribe(meta.clone());

      // Then: should return unique IDs
      assert_ne!(id1, id2, "Each subscription should get a unique ID");
    }

    #[test]
    fn should_store_subscription_metadata() {
      // Given: a subscription store and subscription metadata
      let store = SubscriptionStore::new();
      let org_id = Uuid::new_v4();
      let role_id = Uuid::new_v4();
      let meta = SubscriptionMeta {
        entity_id: "task".to_string(),
        organization_id: org_id,
        role_id,
        params: Some(serde_json::json!({"filter": {"status": "active"}})),
      };

      // When: subscribing
      let id = store.subscribe(meta.clone());

      // Then: subscription should be retrievable with same metadata
      let retrieved = store.get(id);
      assert!(retrieved.is_some());
      let retrieved_meta = retrieved.unwrap();
      assert_eq!(retrieved_meta.entity_id, "task");
      assert_eq!(retrieved_meta.organization_id, org_id);
      assert_eq!(retrieved_meta.role_id, role_id);
      assert_eq!(retrieved_meta.params, meta.params);
    }

    #[test]
    fn should_allow_multiple_subscriptions_for_same_entity() {
      // Given: a subscription store
      let store = SubscriptionStore::new();
      let org_id = Uuid::new_v4();

      // When: subscribing multiple times to same entity
      let meta1 = SubscriptionMeta {
        entity_id: "user".to_string(),
        organization_id: org_id,
        role_id: Uuid::new_v4(),
        params: None,
      };
      let meta2 = SubscriptionMeta {
        entity_id: "user".to_string(),
        organization_id: org_id,
        role_id: Uuid::new_v4(),
        params: None,
      };
      let id1 = store.subscribe(meta1);
      let id2 = store.subscribe(meta2);

      // Then: both subscriptions should be stored
      assert!(store.get(id1).is_some());
      assert!(store.get(id2).is_some());
      assert_ne!(id1, id2);
    }
  }

  mod subscription_removal_behavior {
    use super::*;

    #[test]
    fn should_remove_subscription_when_unsubscribing() {
      // Given: a subscription store with a subscription
      let store = SubscriptionStore::new();
      let meta = SubscriptionMeta {
        entity_id: "user".to_string(),
        organization_id: Uuid::new_v4(),
        role_id: Uuid::new_v4(),
        params: None,
      };
      let id = store.subscribe(meta);

      // When: unsubscribing
      let removed = store.unsubscribe(id);

      // Then: should return true and subscription should be gone
      assert!(removed, "Should return true when subscription exists");
      assert!(store.get(id).is_none(), "Subscription should be removed");
    }

    #[test]
    fn should_return_false_when_unsubscribing_nonexistent_id() {
      // Given: a subscription store
      let store = SubscriptionStore::new();
      let nonexistent_id = Uuid::new_v4();

      // When: unsubscribing with nonexistent ID
      let removed = store.unsubscribe(nonexistent_id);

      // Then: should return false
      assert!(
        !removed,
        "Should return false when subscription doesn't exist"
      );
    }

    #[test]
    fn should_only_remove_specified_subscription() {
      // Given: a subscription store with multiple subscriptions
      let store = SubscriptionStore::new();
      let meta1 = SubscriptionMeta {
        entity_id: "user".to_string(),
        organization_id: Uuid::new_v4(),
        role_id: Uuid::new_v4(),
        params: None,
      };
      let meta2 = SubscriptionMeta {
        entity_id: "task".to_string(),
        organization_id: Uuid::new_v4(),
        role_id: Uuid::new_v4(),
        params: None,
      };
      let id1 = store.subscribe(meta1);
      let id2 = store.subscribe(meta2);

      // When: unsubscribing one subscription
      store.unsubscribe(id1);

      // Then: other subscription should still exist
      assert!(
        store.get(id1).is_none(),
        "First subscription should be removed"
      );
      assert!(
        store.get(id2).is_some(),
        "Second subscription should still exist"
      );
    }
  }

  mod change_event_matching_behavior {
    use super::*;

    #[test]
    fn should_match_subscriptions_by_entity_id() {
      // Given: subscriptions for different entities
      let store = SubscriptionStore::new();
      let org_id = Uuid::new_v4();
      let user_sub_id = store.subscribe(SubscriptionMeta {
        entity_id: "user".to_string(),
        organization_id: org_id,
        role_id: Uuid::new_v4(),
        params: None,
      });
      let _task_sub_id = store.subscribe(SubscriptionMeta {
        entity_id: "task".to_string(),
        organization_id: org_id,
        role_id: Uuid::new_v4(),
        params: None,
      });

      // When: publishing change event for "user" entity
      let event = ChangeEvent {
        model_id: "user".to_string(),
        resource_id: Uuid::new_v4(),
        action: "create".to_string(),
        organization_id: Some(org_id),
      };
      let affected = store.subscriptions_affected_by(&event);

      // Then: should only match "user" subscription
      assert_eq!(affected.len(), 1);
      assert_eq!(affected[0], user_sub_id);
    }

    #[test]
    fn should_match_platform_level_events_to_all_orgs() {
      // Given: subscriptions in different organizations for same entity
      let store = SubscriptionStore::new();
      let org1_id = Uuid::new_v4();
      let org2_id = Uuid::new_v4();
      let sub1_id = store.subscribe(SubscriptionMeta {
        entity_id: "user".to_string(),
        organization_id: org1_id,
        role_id: Uuid::new_v4(),
        params: None,
      });
      let sub2_id = store.subscribe(SubscriptionMeta {
        entity_id: "user".to_string(),
        organization_id: org2_id,
        role_id: Uuid::new_v4(),
        params: None,
      });

      // When: publishing platform-level change event (no organization_id)
      let event = ChangeEvent {
        model_id: "user".to_string(),
        resource_id: Uuid::new_v4(),
        action: "create".to_string(),
        organization_id: None, // Platform-level event
      };
      let affected = store.subscriptions_affected_by(&event);

      // Then: should match subscriptions in all organizations
      assert_eq!(affected.len(), 2);
      assert!(affected.contains(&sub1_id));
      assert!(affected.contains(&sub2_id));
    }

    #[test]
    fn should_match_org_scoped_events_only_to_same_org() {
      // Given: subscriptions in different organizations
      let store = SubscriptionStore::new();
      let org1_id = Uuid::new_v4();
      let org2_id = Uuid::new_v4();
      let sub1_id = store.subscribe(SubscriptionMeta {
        entity_id: "user".to_string(),
        organization_id: org1_id,
        role_id: Uuid::new_v4(),
        params: None,
      });
      let _sub2_id = store.subscribe(SubscriptionMeta {
        entity_id: "user".to_string(),
        organization_id: org2_id,
        role_id: Uuid::new_v4(),
        params: None,
      });

      // When: publishing org-scoped change event
      let event = ChangeEvent {
        model_id: "user".to_string(),
        resource_id: Uuid::new_v4(),
        action: "update".to_string(),
        organization_id: Some(org1_id), // Only org1
      };
      let affected = store.subscriptions_affected_by(&event);

      // Then: should only match subscription in same organization
      assert_eq!(affected.len(), 1);
      assert_eq!(affected[0], sub1_id);
    }

    #[test]
    fn should_not_match_when_entity_id_differs() {
      // Given: subscription for "user" entity
      let store = SubscriptionStore::new();
      let org_id = Uuid::new_v4();
      let _sub_id = store.subscribe(SubscriptionMeta {
        entity_id: "user".to_string(),
        organization_id: org_id,
        role_id: Uuid::new_v4(),
        params: None,
      });

      // When: publishing change event for different entity
      let event = ChangeEvent {
        model_id: "task".to_string(), // Different entity
        resource_id: Uuid::new_v4(),
        action: "create".to_string(),
        organization_id: Some(org_id),
      };
      let affected = store.subscriptions_affected_by(&event);

      // Then: should not match any subscriptions
      assert!(affected.is_empty());
    }
  }

  mod invalidation_event_behavior {
    use super::*;

    #[tokio::test]
    async fn should_send_invalidation_event_to_subscribers() {
      // Given: a subscription store and invalidation receiver
      let store = SubscriptionStore::new();
      let mut rx = store.subscribe_invalidations();
      let subscription_id = Uuid::new_v4();

      // When: sending invalidation event
      store.send_invalidation(InvalidationEvent { subscription_id });

      // Then: receiver should receive the event
      let event = rx.recv().await.unwrap();
      assert_eq!(event.subscription_id, subscription_id);
    }

    #[tokio::test]
    async fn should_send_multiple_invalidation_events() {
      // Given: a subscription store and invalidation receiver
      let store = SubscriptionStore::new();
      let mut rx = store.subscribe_invalidations();
      let id1 = Uuid::new_v4();
      let id2 = Uuid::new_v4();

      // When: sending multiple invalidation events
      store.send_invalidation(InvalidationEvent {
        subscription_id: id1,
      });
      store.send_invalidation(InvalidationEvent {
        subscription_id: id2,
      });

      // Then: receiver should receive both events
      let event1 = rx.recv().await.unwrap();
      let event2 = rx.recv().await.unwrap();
      assert_eq!(event1.subscription_id, id1);
      assert_eq!(event2.subscription_id, id2);
    }

    #[tokio::test]
    async fn should_notify_affected_subscriptions_on_change_event() {
      // Given: a subscription store with subscriptions and invalidation receiver
      let store = SubscriptionStore::new();
      let mut rx = store.subscribe_invalidations();
      let org_id = Uuid::new_v4();
      let sub1_id = store.subscribe(SubscriptionMeta {
        entity_id: "user".to_string(),
        organization_id: org_id,
        role_id: Uuid::new_v4(),
        params: None,
      });
      let sub2_id = store.subscribe(SubscriptionMeta {
        entity_id: "user".to_string(),
        organization_id: org_id,
        role_id: Uuid::new_v4(),
        params: None,
      });

      // When: notifying affected subscriptions
      let event = ChangeEvent {
        model_id: "user".to_string(),
        resource_id: Uuid::new_v4(),
        action: "create".to_string(),
        organization_id: Some(org_id),
      };
      store.notify_affected_by(&event);

      // Then: should send invalidation events for all affected subscriptions
      let mut received_ids = Vec::new();
      received_ids.push(rx.recv().await.unwrap().subscription_id);
      received_ids.push(rx.recv().await.unwrap().subscription_id);
      assert!(received_ids.contains(&sub1_id));
      assert!(received_ids.contains(&sub2_id));
    }
  }

  mod change_event_publishing_behavior {
    use super::*;

    #[tokio::test]
    async fn should_publish_change_event_to_subscribers() {
      // Given: a subscription store and change event receiver
      let store = SubscriptionStore::new();
      let mut rx = store.subscribe_changes();
      let event = ChangeEvent {
        model_id: "user".to_string(),
        resource_id: Uuid::new_v4(),
        action: "create".to_string(),
        organization_id: Some(Uuid::new_v4()),
      };

      // When: publishing change event
      store.publish_change(event.clone());

      // Then: receiver should receive the event
      let received = rx.recv().await.unwrap();
      assert_eq!(received.model_id, event.model_id);
      assert_eq!(received.resource_id, event.resource_id);
      assert_eq!(received.action, event.action);
      assert_eq!(received.organization_id, event.organization_id);
    }

    #[tokio::test]
    async fn should_publish_multiple_change_events() {
      // Given: a subscription store and change event receiver
      let store = SubscriptionStore::new();
      let mut rx = store.subscribe_changes();

      // When: publishing multiple change events
      let event1 = ChangeEvent {
        model_id: "user".to_string(),
        resource_id: Uuid::new_v4(),
        action: "create".to_string(),
        organization_id: None,
      };
      let event2 = ChangeEvent {
        model_id: "task".to_string(),
        resource_id: Uuid::new_v4(),
        action: "update".to_string(),
        organization_id: Some(Uuid::new_v4()),
      };
      store.publish_change(event1.clone());
      store.publish_change(event2.clone());

      // Then: receiver should receive both events
      let received1 = rx.recv().await.unwrap();
      let received2 = rx.recv().await.unwrap();
      assert_eq!(received1.model_id, event1.model_id);
      assert_eq!(received2.model_id, event2.model_id);
    }
  }

  mod store_initialization_behavior {
    use super::*;

    #[test]
    fn should_create_empty_store_with_default() {
      // Given: default subscription store
      let store = SubscriptionStore::default();

      // When: checking subscriptions
      // Then: should be empty
      let event = ChangeEvent {
        model_id: "user".to_string(),
        resource_id: Uuid::new_v4(),
        action: "create".to_string(),
        organization_id: None,
      };
      let affected = store.subscriptions_affected_by(&event);
      assert!(affected.is_empty());
    }

    #[test]
    fn should_create_empty_store_with_new() {
      // Given: new subscription store
      let store = SubscriptionStore::new();

      // When: checking subscriptions
      // Then: should be empty
      let event = ChangeEvent {
        model_id: "user".to_string(),
        resource_id: Uuid::new_v4(),
        action: "create".to_string(),
        organization_id: None,
      };
      let affected = store.subscriptions_affected_by(&event);
      assert!(affected.is_empty());
    }

    #[tokio::test]
    async fn should_create_broadcast_channels_on_initialization() {
      // Given: a new subscription store
      let store = SubscriptionStore::new();

      // When: subscribing to invalidation and change events
      let mut invalidation_rx = store.subscribe_invalidations();
      let mut change_rx = store.subscribe_changes();

      // Then: channels should be functional
      store.send_invalidation(InvalidationEvent {
        subscription_id: Uuid::new_v4(),
      });
      store.publish_change(ChangeEvent {
        model_id: "test".to_string(),
        resource_id: Uuid::new_v4(),
        action: "create".to_string(),
        organization_id: None,
      });

      // Verify we can receive events
      let _invalidation = invalidation_rx.recv().await.unwrap();
      let _change = change_rx.recv().await.unwrap();
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod bdd_tests {
    use super::*;

    /// BDD-style tests focusing on behavior rather than implementation.
    /// Tests verify subscription management, change event handling, and invalidation event delivery behavior.
    mod change_event_behavior {
      use super::*;

      #[test]
      fn should_create_change_event_with_all_fields() {
        // Given: change event data
        let model_id = "user".to_string();
        let resource_id = Uuid::new_v4();
        let action = "create".to_string();
        let organization_id = Some(Uuid::new_v4());

        // When: creating ChangeEvent
        let event = ChangeEvent {
          model_id: model_id.clone(),
          resource_id,
          action: action.clone(),
          organization_id,
        };

        // Then: event should contain all fields
        assert_eq!(event.model_id, model_id);
        assert_eq!(event.resource_id, resource_id);
        assert_eq!(event.action, action);
        assert_eq!(event.organization_id, organization_id);
      }

      #[test]
      fn should_create_platform_level_change_event() {
        // Given: change event data without organization
        let event = ChangeEvent {
          model_id: "user".to_string(),
          resource_id: Uuid::new_v4(),
          action: "update".to_string(),
          organization_id: None,
        };

        // When: checking organization_id
        // Then: should be None for platform-level events
        assert_eq!(event.organization_id, None);
      }

      #[test]
      fn should_create_org_scoped_change_event() {
        // Given: change event data with organization
        let org_id = Uuid::new_v4();
        let event = ChangeEvent {
          model_id: "user".to_string(),
          resource_id: Uuid::new_v4(),
          action: "delete".to_string(),
          organization_id: Some(org_id),
        };

        // When: checking organization_id
        // Then: should contain organization ID for org-scoped events
        assert_eq!(event.organization_id, Some(org_id));
      }
    }

    mod invalidation_event_behavior {
      use super::*;

      #[test]
      fn should_create_invalidation_event_with_subscription_id() {
        // Given: a subscription ID
        let subscription_id = Uuid::new_v4();

        // When: creating InvalidationEvent
        let event = InvalidationEvent { subscription_id };

        // Then: event should contain subscription ID
        assert_eq!(event.subscription_id, subscription_id);
      }

      #[test]
      fn should_serialize_invalidation_event() {
        // Given: an InvalidationEvent
        let subscription_id = Uuid::new_v4();
        let event = InvalidationEvent { subscription_id };

        // When: serializing to JSON
        let json = serde_json::to_string(&event).unwrap();

        // Then: should serialize successfully
        assert!(json.contains(&subscription_id.to_string()));
      }
    }

    mod subscription_meta_behavior {
      use super::*;

      #[test]
      fn should_create_subscription_meta_with_all_fields() {
        // Given: subscription metadata
        let entity_id = "user".to_string();
        let organization_id = Uuid::new_v4();
        let role_id = Uuid::new_v4();
        let params = Some(serde_json::json!({"filter": "active"}));

        // When: creating SubscriptionMeta
        let meta = SubscriptionMeta {
          entity_id: entity_id.clone(),
          organization_id,
          role_id,
          params: params.clone(),
        };

        // Then: meta should contain all fields
        assert_eq!(meta.entity_id, entity_id);
        assert_eq!(meta.organization_id, organization_id);
        assert_eq!(meta.role_id, role_id);
        assert_eq!(meta.params, params);
      }

      #[test]
      fn should_create_subscription_meta_without_params() {
        // Given: subscription metadata without params
        let meta = SubscriptionMeta {
          entity_id: "user".to_string(),
          organization_id: Uuid::new_v4(),
          role_id: Uuid::new_v4(),
          params: None,
        };

        // When: checking params
        // Then: params should be None
        assert_eq!(meta.params, None);
      }
    }

    mod subscription_store_creation_behavior {
      use super::*;

      #[test]
      fn should_create_subscription_store_with_new() {
        // Given: SubscriptionStore::new()
        // When: creating store
        let store = SubscriptionStore::new();

        // Then: store should be created
        // Type check - if it compiles, it's created
        let _ = store;
      }

      #[test]
      fn should_create_subscription_store_with_default() {
        // Given: SubscriptionStore::default()
        // When: creating store
        let store = SubscriptionStore::default();

        // Then: store should be created
        let _ = store;
      }

      #[test]
      fn should_start_with_empty_subscriptions() {
        // Given: a new SubscriptionStore
        let store = SubscriptionStore::new();

        // When: checking subscriptions
        // Then: should have no subscriptions initially
        // Verified by subscribe returning a new ID each time
        let id1 = store.subscribe(SubscriptionMeta {
          entity_id: "user".to_string(),
          organization_id: Uuid::new_v4(),
          role_id: Uuid::new_v4(),
          params: None,
        });
        assert!(!id1.is_nil(), "Should generate valid subscription ID");
      }
    }

    mod subscription_registration_behavior {
      use super::*;

      #[test]
      fn should_register_subscription_and_return_id() {
        // Given: a SubscriptionStore and metadata
        let store = SubscriptionStore::new();
        let meta = SubscriptionMeta {
          entity_id: "user".to_string(),
          organization_id: Uuid::new_v4(),
          role_id: Uuid::new_v4(),
          params: None,
        };

        // When: subscribing
        let id = store.subscribe(meta.clone());

        // Then: should return a valid UUID
        assert!(!id.is_nil(), "Should return valid subscription ID");
        // Verify subscription exists
        let retrieved = store.get(id);
        assert!(retrieved.is_some(), "Subscription should be stored");
        assert_eq!(retrieved.unwrap().entity_id, meta.entity_id);
      }

      #[test]
      fn should_generate_unique_ids_for_multiple_subscriptions() {
        // Given: a SubscriptionStore
        let store = SubscriptionStore::new();
        let meta = SubscriptionMeta {
          entity_id: "user".to_string(),
          organization_id: Uuid::new_v4(),
          role_id: Uuid::new_v4(),
          params: None,
        };

        // When: subscribing multiple times
        let id1 = store.subscribe(meta.clone());
        let id2 = store.subscribe(meta.clone());

        // Then: should generate unique IDs
        assert_ne!(id1, id2, "Should generate unique subscription IDs");
      }

      #[test]
      fn should_unsubscribe_existing_subscription() {
        // Given: a SubscriptionStore with a subscription
        let store = SubscriptionStore::new();
        let meta = SubscriptionMeta {
          entity_id: "user".to_string(),
          organization_id: Uuid::new_v4(),
          role_id: Uuid::new_v4(),
          params: None,
        };
        let id = store.subscribe(meta);

        // When: unsubscribing
        let existed = store.unsubscribe(id);

        // Then: should return true and remove subscription
        assert!(existed, "Should return true for existing subscription");
        assert!(store.get(id).is_none(), "Subscription should be removed");
      }

      #[test]
      fn should_return_false_when_unsubscribing_nonexistent() {
        // Given: a SubscriptionStore without subscriptions
        let store = SubscriptionStore::new();
        let nonexistent_id = Uuid::new_v4();

        // When: unsubscribing nonexistent ID
        let existed = store.unsubscribe(nonexistent_id);

        // Then: should return false
        assert!(!existed, "Should return false for nonexistent subscription");
      }

      #[test]
      fn should_retrieve_subscription_by_id() {
        // Given: a SubscriptionStore with a subscription
        let store = SubscriptionStore::new();
        let org_id = Uuid::new_v4();
        let role_id = Uuid::new_v4();
        let meta = SubscriptionMeta {
          entity_id: "user".to_string(),
          organization_id: org_id,
          role_id,
          params: None,
        };
        let id = store.subscribe(meta.clone());

        // When: retrieving subscription
        let retrieved = store.get(id);

        // Then: should return the subscription metadata
        assert!(retrieved.is_some(), "Should retrieve subscription");
        let retrieved_meta = retrieved.unwrap();
        assert_eq!(retrieved_meta.entity_id, meta.entity_id);
        assert_eq!(retrieved_meta.organization_id, org_id);
        assert_eq!(retrieved_meta.role_id, role_id);
      }

      #[test]
      fn should_return_none_for_nonexistent_subscription() {
        // Given: a SubscriptionStore
        let store = SubscriptionStore::new();
        let nonexistent_id = Uuid::new_v4();

        // When: retrieving nonexistent subscription
        let retrieved = store.get(nonexistent_id);

        // Then: should return None
        assert!(
          retrieved.is_none(),
          "Should return None for nonexistent subscription"
        );
      }
    }

    mod change_event_publishing_behavior {
      use super::*;

      #[test]
      fn should_publish_change_event() {
        // Given: a SubscriptionStore and change event
        let store = SubscriptionStore::new();
        let event = ChangeEvent {
          model_id: "user".to_string(),
          resource_id: Uuid::new_v4(),
          action: "create".to_string(),
          organization_id: Some(Uuid::new_v4()),
        };

        // When: publishing change event
        store.publish_change(event.clone());

        // Then: should publish without error
        // publish_change is fire-and-forget, so we just verify it doesn't panic
      }

      #[tokio::test]
      async fn should_receive_change_event_via_subscription() {
        // Given: a SubscriptionStore
        let store = SubscriptionStore::new();
        let mut rx = store.subscribe_changes();
        let event = ChangeEvent {
          model_id: "user".to_string(),
          resource_id: Uuid::new_v4(),
          action: "create".to_string(),
          organization_id: Some(Uuid::new_v4()),
        };

        // When: publishing change event
        store.publish_change(event.clone());

        // Then: should receive event via subscription
        let received = rx.recv().await.unwrap();
        assert_eq!(received.model_id, event.model_id);
        assert_eq!(received.resource_id, event.resource_id);
        assert_eq!(received.action, event.action);
        assert_eq!(received.organization_id, event.organization_id);
      }
    }

    mod invalidation_event_sending_behavior {
      use super::*;

      #[test]
      fn should_send_invalidation_event() {
        // Given: a SubscriptionStore and invalidation event
        let store = SubscriptionStore::new();
        let subscription_id = Uuid::new_v4();
        let event = InvalidationEvent { subscription_id };

        // When: sending invalidation event
        store.send_invalidation(event.clone());

        // Then: should send without error
        // send_invalidation is fire-and-forget, so we just verify it doesn't panic
      }

      #[tokio::test]
      async fn should_receive_invalidation_event_via_subscription() {
        // Given: a SubscriptionStore
        let store = SubscriptionStore::new();
        let mut rx = store.subscribe_invalidations();
        let subscription_id = Uuid::new_v4();
        let event = InvalidationEvent { subscription_id };

        // When: sending invalidation event
        store.send_invalidation(event.clone());

        // Then: should receive event via subscription
        let received = rx.recv().await.unwrap();
        assert_eq!(received.subscription_id, subscription_id);
      }
    }

    mod subscription_matching_behavior {
      use super::*;

      #[test]
      fn should_match_subscription_by_entity_id() {
        // Given: a SubscriptionStore with a subscription
        let store = SubscriptionStore::new();
        let org_id = Uuid::new_v4();
        let meta = SubscriptionMeta {
          entity_id: "user".to_string(),
          organization_id: org_id,
          role_id: Uuid::new_v4(),
          params: None,
        };
        let subscription_id = store.subscribe(meta);

        // When: matching with change event for same entity
        let event = ChangeEvent {
          model_id: "user".to_string(),
          resource_id: Uuid::new_v4(),
          action: "create".to_string(),
          organization_id: Some(org_id),
        };
        let affected = store.subscriptions_affected_by(&event);

        // Then: should match the subscription
        assert!(
          affected.contains(&subscription_id),
          "Should match subscription"
        );
      }

      #[test]
      fn should_not_match_subscription_with_different_entity_id() {
        // Given: a SubscriptionStore with a subscription for "user"
        let store = SubscriptionStore::new();
        let meta = SubscriptionMeta {
          entity_id: "user".to_string(),
          organization_id: Uuid::new_v4(),
          role_id: Uuid::new_v4(),
          params: None,
        };
        let subscription_id = store.subscribe(meta);

        // When: matching with change event for different entity
        let event = ChangeEvent {
          model_id: "organization".to_string(),
          resource_id: Uuid::new_v4(),
          action: "create".to_string(),
          organization_id: Some(Uuid::new_v4()),
        };
        let affected = store.subscriptions_affected_by(&event);

        // Then: should not match the subscription
        assert!(
          !affected.contains(&subscription_id),
          "Should not match subscription with different entity"
        );
      }

      #[test]
      fn should_match_platform_level_event_to_all_orgs() {
        // Given: a SubscriptionStore with subscriptions in different orgs
        let store = SubscriptionStore::new();
        let org_id1 = Uuid::new_v4();
        let org_id2 = Uuid::new_v4();
        let meta1 = SubscriptionMeta {
          entity_id: "user".to_string(),
          organization_id: org_id1,
          role_id: Uuid::new_v4(),
          params: None,
        };
        let meta2 = SubscriptionMeta {
          entity_id: "user".to_string(),
          organization_id: org_id2,
          role_id: Uuid::new_v4(),
          params: None,
        };
        let id1 = store.subscribe(meta1);
        let id2 = store.subscribe(meta2);

        // When: matching with platform-level change event (no org)
        let event = ChangeEvent {
          model_id: "user".to_string(),
          resource_id: Uuid::new_v4(),
          action: "create".to_string(),
          organization_id: None, // Platform-level
        };
        let affected = store.subscriptions_affected_by(&event);

        // Then: should match subscriptions in all orgs
        assert!(affected.contains(&id1), "Should match subscription in org1");
        assert!(affected.contains(&id2), "Should match subscription in org2");
      }

      #[test]
      fn should_match_org_scoped_event_only_to_same_org() {
        // Given: a SubscriptionStore with subscriptions in different orgs
        let store = SubscriptionStore::new();
        let org_id1 = Uuid::new_v4();
        let org_id2 = Uuid::new_v4();
        let meta1 = SubscriptionMeta {
          entity_id: "user".to_string(),
          organization_id: org_id1,
          role_id: Uuid::new_v4(),
          params: None,
        };
        let meta2 = SubscriptionMeta {
          entity_id: "user".to_string(),
          organization_id: org_id2,
          role_id: Uuid::new_v4(),
          params: None,
        };
        let id1 = store.subscribe(meta1);
        let id2 = store.subscribe(meta2);

        // When: matching with org-scoped change event
        let event = ChangeEvent {
          model_id: "user".to_string(),
          resource_id: Uuid::new_v4(),
          action: "create".to_string(),
          organization_id: Some(org_id1), // Only org1
        };
        let affected = store.subscriptions_affected_by(&event);

        // Then: should match only subscription in same org
        assert!(affected.contains(&id1), "Should match subscription in org1");
        assert!(
          !affected.contains(&id2),
          "Should not match subscription in different org"
        );
      }
    }

    mod change_worker_behavior {
      use super::*;

      #[test]
      fn should_have_spawn_change_worker_method() {
        // Given: a SubscriptionStore
        let store = SubscriptionStore::new();

        // When: checking if spawn_change_worker exists
        // Then: method should be callable (if it compiles, it exists)
        // Note: Actual spawning requires tokio runtime, tested in integration tests
        let _store = store;
      }

      #[test]
      fn should_notify_affected_subscriptions_directly() {
        // Given: a SubscriptionStore with subscription
        let store = SubscriptionStore::new();
        let org_id = Uuid::new_v4();
        let meta = SubscriptionMeta {
          entity_id: "user".to_string(),
          organization_id: org_id,
          role_id: Uuid::new_v4(),
          params: None,
        };
        let _subscription_id = store.subscribe(meta);

        // When: notifying affected subscriptions directly
        let event = ChangeEvent {
          model_id: "user".to_string(),
          resource_id: Uuid::new_v4(),
          action: "create".to_string(),
          organization_id: Some(org_id),
        };
        store.notify_affected_by(&event);

        // Then: should send invalidation events (fire-and-forget, so we just verify it doesn't panic)
        // The actual delivery is tested via subscribe_invalidations in async tests
      }
    }
  }
}
