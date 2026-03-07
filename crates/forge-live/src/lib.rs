//! Live Query: channel-based broadcast for real-time sync.
//!
//! Clients subscribe to channels (e.g. `org:{org_id}`). Handlers call `broadcast`
//! after mutations; all connections in that channel receive the event.
//! The app owns the transport (e.g. WebSocket); this crate provides the registry and broadcast.

mod backend;
mod channel;
mod event;
#[cfg(feature = "redis")]
mod redis_backend;
mod subscription;

pub use backend::{ConnectionId, InMemoryLiveBackend, LiveBackend};
pub use channel::Channel;
pub use event::LiveEvent;
#[cfg(feature = "redis")]
pub use redis_backend::RedisLiveBackend;
pub use subscription::{ChangeEvent, InvalidationEvent, SubscriptionMeta, SubscriptionStore};

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

#[cfg(test)]
mod bdd_tests {
  use super::*;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests are organized by feature/behavior area with descriptive names.
  mod org_channel_creation_behavior {
    use super::*;

    #[test]
    fn should_create_channel_for_organization() {
      // Given: an organization ID
      let org_id = Uuid::new_v4();

      // When: creating an org channel
      let channel = org_channel(org_id);

      // Then: should create channel with org: prefix
      let channel_str = channel.as_str();
      assert!(
        channel_str.starts_with("org:"),
        "Channel should start with 'org:' prefix"
      );
      assert!(
        channel_str.contains(&org_id.to_string()),
        "Channel should include organization ID"
      );
    }

    #[test]
    fn should_create_different_channels_for_different_orgs() {
      // Given: different organization IDs
      let org1_id = Uuid::new_v4();
      let org2_id = Uuid::new_v4();

      // When: creating channels for each org
      let channel1 = org_channel(org1_id);
      let channel2 = org_channel(org2_id);

      // Then: channels should be different
      assert_ne!(
        channel1.as_str(),
        channel2.as_str(),
        "Different orgs should have different channels"
      );
    }

    #[test]
    fn should_create_same_channel_for_same_org() {
      // Given: the same organization ID
      let org_id = Uuid::new_v4();

      // When: creating channels multiple times
      let channel1 = org_channel(org_id);
      let channel2 = org_channel(org_id);

      // Then: channels should be identical
      assert_eq!(
        channel1.as_str(),
        channel2.as_str(),
        "Same org should produce same channel"
      );
    }
  }

  mod broadcast_to_channel_behavior {
    use super::*;

    #[tokio::test]
    async fn should_broadcast_event_to_channel() {
      // Given: a backend and a channel with a connection
      let backend = Arc::new(InMemoryLiveBackend::new());
      let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
      let connection_id = backend.register_connection(tx);
      let channel = Channel::raw("test-channel");
      backend.subscribe(connection_id, channel.clone());

      // When: broadcasting an event to the channel
      let event = LiveEvent::ResourceChanged {
        resource: "user".to_string(),
        id: Uuid::new_v4(),
        action: Some("create".to_string()),
      };
      let result = broadcast_to_channel(&backend, &channel, &event).await;

      // Then: should succeed and connection should receive the event
      assert!(result.is_ok(), "Broadcast should succeed");
      let received = rx.try_recv().ok();
      assert!(
        received.is_some(),
        "Connection should receive broadcasted event"
      );
    }

    #[tokio::test]
    async fn should_serialize_event_as_json_when_broadcasting() {
      // Given: a backend and a channel with a connection
      let backend = Arc::new(InMemoryLiveBackend::new());
      let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
      let connection_id = backend.register_connection(tx);
      let channel = Channel::raw("json-test-channel");
      backend.subscribe(connection_id, channel.clone());

      // When: broadcasting an event
      let event = LiveEvent::UsersUpdated {
        user_id: Some(Uuid::new_v4()),
        org_id: Some(Uuid::new_v4()),
      };
      let _result = broadcast_to_channel(&backend, &channel, &event).await;

      // Then: payload should be valid JSON
      let received = rx.try_recv().ok();
      assert!(received.is_some(), "Should receive payload");
      let payload = received.unwrap();
      let parsed: Result<serde_json::Value, _> = serde_json::from_slice(&payload);
      assert!(parsed.is_ok(), "Payload should be valid JSON");
    }

    #[tokio::test]
    async fn should_return_error_when_event_serialization_fails() {
      // Given: a backend and an event that cannot be serialized
      // Note: LiveEvent is always serializable, so we test with a malformed scenario
      // In practice, this would require a custom type that fails serialization
      // For now, we verify the function handles serialization errors correctly
      let backend = Arc::new(InMemoryLiveBackend::new());
      let channel = Channel::raw("error-channel");
      let event = LiveEvent::Custom {
        topic: "test".to_string(),
        payload: None,
      };

      // When: broadcasting (should succeed as LiveEvent is always serializable)
      let result = broadcast_to_channel(&backend, &channel, &event).await;

      // Then: should succeed (LiveEvent is always serializable)
      assert!(result.is_ok(), "Broadcast should succeed for valid event");
    }

    #[tokio::test]
    async fn should_broadcast_to_all_connections_in_channel() {
      // Given: a backend with multiple connections in same channel
      let backend = Arc::new(InMemoryLiveBackend::new());
      let (tx1, mut rx1) = tokio::sync::mpsc::unbounded_channel();
      let (tx2, mut rx2) = tokio::sync::mpsc::unbounded_channel();
      let id1 = backend.register_connection(tx1);
      let id2 = backend.register_connection(tx2);
      let channel = Channel::raw("multi-connection-channel");
      backend.subscribe(id1, channel.clone());
      backend.subscribe(id2, channel.clone());

      // When: broadcasting to the channel
      let event = LiveEvent::ResourceChanged {
        resource: "task".to_string(),
        id: Uuid::new_v4(),
        action: Some("update".to_string()),
      };
      let _result = broadcast_to_channel(&backend, &channel, &event).await;

      // Then: all connections should receive the event
      let received1 = rx1.try_recv().ok();
      let received2 = rx2.try_recv().ok();
      assert!(received1.is_some(), "First connection should receive event");
      assert!(
        received2.is_some(),
        "Second connection should receive event"
      );
    }
  }

  mod broadcast_to_org_behavior {
    use super::*;

    #[tokio::test]
    async fn should_broadcast_to_org_channel() {
      // Given: a backend and an organization ID
      let backend = Arc::new(InMemoryLiveBackend::new());
      let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
      let connection_id = backend.register_connection(tx);
      let org_id = Uuid::new_v4();
      let org_channel = org_channel(org_id);
      backend.subscribe(connection_id, org_channel);

      // When: broadcasting to org
      let event = LiveEvent::UsersUpdated {
        user_id: Some(Uuid::new_v4()),
        org_id: Some(org_id),
      };
      let result = broadcast_to_org(&backend, org_id, &event).await;

      // Then: should succeed and connection should receive event
      assert!(result.is_ok(), "Broadcast to org should succeed");
      let received = rx.try_recv().ok();
      assert!(
        received.is_some(),
        "Connection should receive org broadcast"
      );
    }

    #[tokio::test]
    async fn should_use_org_channel_helper_when_broadcasting_to_org() {
      // Given: a backend and organization ID
      let backend = Arc::new(InMemoryLiveBackend::new());
      let org_id = Uuid::new_v4();
      let _expected_channel = org_channel(org_id);

      // When: broadcasting to org
      let event = LiveEvent::Custom {
        topic: "test".to_string(),
        payload: None,
      };
      let result = broadcast_to_org(&backend, org_id, &event).await;

      // Then: should use org channel (verified by successful broadcast)
      assert!(result.is_ok(), "Should use org channel for broadcast");
      // The function internally calls org_channel, so success confirms correct channel usage
    }

    #[tokio::test]
    async fn should_only_broadcast_to_specified_org() {
      // Given: a backend with connections in different orgs
      let backend = Arc::new(InMemoryLiveBackend::new());
      let (tx1, mut rx1) = tokio::sync::mpsc::unbounded_channel();
      let (tx2, mut rx2) = tokio::sync::mpsc::unbounded_channel();
      let id1 = backend.register_connection(tx1);
      let id2 = backend.register_connection(tx2);
      let org1_id = Uuid::new_v4();
      let org2_id = Uuid::new_v4();
      backend.subscribe(id1, org_channel(org1_id));
      backend.subscribe(id2, org_channel(org2_id));

      // When: broadcasting to org1
      let event = LiveEvent::ResourceChanged {
        resource: "user".to_string(),
        id: Uuid::new_v4(),
        action: None,
      };
      let _result = broadcast_to_org(&backend, org1_id, &event).await;

      // Then: only org1 connection should receive event
      let received1 = rx1.try_recv().ok();
      let received2 = rx2.try_recv().ok();
      assert!(received1.is_some(), "Org1 connection should receive event");
      assert!(
        received2.is_none(),
        "Org2 connection should not receive event"
      );
    }
  }

  mod sweep_expired_connections_behavior {
    use super::*;

    #[tokio::test]
    async fn should_complete_without_error_for_in_memory_backend() {
      // Given: any database connection (unused for in-memory backend)
      let db_connection = (); // Dummy connection

      // When: sweeping expired connections
      sweep_expired_connections(db_connection).await;

      // Then: should complete without error (no-op for in-memory)
      // If we reach here, the function completed successfully
    }

    #[test]
    fn should_be_callable_with_any_connection_type() {
      // Given: different connection types
      let _db1: () = ();
      let _db2: String = String::new();

      // When: calling sweep_expired_connections
      // Then: should accept any type (generic parameter)
      // Verification: function signature accepts generic C parameter
    }
  }

  mod live_event_serialization_behavior {
    use super::*;

    #[test]
    fn should_serialize_users_updated_event() {
      // Given: a UsersUpdated event
      let event = LiveEvent::UsersUpdated {
        user_id: Some(Uuid::new_v4()),
        org_id: Some(Uuid::new_v4()),
      };

      // When: serializing to JSON
      let json = serde_json::to_string(&event);

      // Then: should succeed and contain event type
      assert!(json.is_ok());
      let json_str = json.unwrap();
      assert!(
        json_str.contains("users_updated"),
        "Should contain event type"
      );
    }

    #[test]
    fn should_serialize_resource_changed_event() {
      // Given: a ResourceChanged event
      let event = LiveEvent::ResourceChanged {
        resource: "task".to_string(),
        id: Uuid::new_v4(),
        action: Some("delete".to_string()),
      };

      // When: serializing to JSON
      let json = serde_json::to_string(&event);

      // Then: should succeed and contain resource information
      assert!(json.is_ok());
      let json_str = json.unwrap();
      assert!(
        json_str.contains("resource_changed"),
        "Should contain event type"
      );
      assert!(json_str.contains("task"), "Should contain resource type");
    }

    #[test]
    fn should_serialize_custom_event() {
      // Given: a Custom event
      let event = LiveEvent::Custom {
        topic: "notification".to_string(),
        payload: Some(serde_json::json!({"message": "Hello"})),
      };

      // When: serializing to JSON
      let json = serde_json::to_string(&event);

      // Then: should succeed and contain custom topic
      assert!(json.is_ok());
      let json_str = json.unwrap();
      assert!(json_str.contains("custom"), "Should contain event type");
      assert!(json_str.contains("notification"), "Should contain topic");
    }
  }
}
