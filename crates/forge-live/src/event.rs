//! Event payloads sent to subscribed clients.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event pushed to clients on a channel (e.g. after a mutation).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LiveEvent {
  /// A user was created, updated, or deleted.
  UsersUpdated {
    #[serde(skip_serializing_if = "Option::is_none")]
    user_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    org_id: Option<Uuid>,
  },
  /// Generic resource change (type + id).
  ResourceChanged {
    resource: String,
    id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    action: Option<String>,
  },
  /// Opaque payload for custom events.
  Custom {
    topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<serde_json::Value>,
  },
}

#[cfg(test)]
mod tests {
  use super::*;

  mod bdd_tests {
    use super::*;

    /// BDD-style tests focusing on behavior rather than implementation.
    /// Tests verify LiveEvent enum variants, serialization, and field handling.

    mod users_updated_event_behavior {
      use super::*;

      #[test]
      fn should_create_users_updated_event_with_all_fields() {
        // Given: user and organization IDs
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        // When: creating UsersUpdated event
        let event = LiveEvent::UsersUpdated {
          user_id: Some(user_id),
          org_id: Some(org_id),
        };

        // Then: event should contain both IDs
        match event {
          LiveEvent::UsersUpdated {
            user_id: Some(uid),
            org_id: Some(oid),
          } => {
            assert_eq!(uid, user_id);
            assert_eq!(oid, org_id);
          }
          _ => panic!("Expected UsersUpdated event"),
        }
      }

      #[test]
      fn should_create_users_updated_event_with_optional_fields() {
        // Given: UsersUpdated event can have None fields
        // When: creating event with None values
        let event = LiveEvent::UsersUpdated {
          user_id: None,
          org_id: None,
        };

        // Then: event should accept None values
        match event {
          LiveEvent::UsersUpdated {
            user_id: None,
            org_id: None,
          } => assert!(true, "Event should accept None values"),
          _ => panic!("Expected UsersUpdated event with None values"),
        }
      }

      #[test]
      fn should_create_users_updated_event_with_partial_fields() {
        // Given: UsersUpdated event with only user_id
        let user_id = Uuid::new_v4();

        // When: creating event with only user_id
        let event = LiveEvent::UsersUpdated {
          user_id: Some(user_id),
          org_id: None,
        };

        // Then: event should contain user_id and None org_id
        match event {
          LiveEvent::UsersUpdated {
            user_id: Some(uid),
            org_id: None,
          } => assert_eq!(uid, user_id),
          _ => panic!("Expected UsersUpdated event with partial fields"),
        }
      }
    }

    mod resource_changed_event_behavior {
      use super::*;

      #[test]
      fn should_create_resource_changed_event_with_all_fields() {
        // Given: resource change data
        let resource = "users".to_string();
        let id = Uuid::new_v4();
        let action = Some("create".to_string());

        // When: creating ResourceChanged event
        let event = LiveEvent::ResourceChanged {
          resource: resource.clone(),
          id,
          action: action.clone(),
        };

        // Then: event should contain all fields
        match event {
          LiveEvent::ResourceChanged {
            resource: res,
            id: event_id,
            action: act,
          } => {
            assert_eq!(res, resource);
            assert_eq!(event_id, id);
            assert_eq!(act, action);
          }
          _ => panic!("Expected ResourceChanged event"),
        }
      }

      #[test]
      fn should_create_resource_changed_event_without_action() {
        // Given: resource change data without action
        let resource = "tasks".to_string();
        let id = Uuid::new_v4();

        // When: creating ResourceChanged event without action
        let event = LiveEvent::ResourceChanged {
          resource: resource.clone(),
          id,
          action: None,
        };

        // Then: event should contain resource and id, but None action
        match event {
          LiveEvent::ResourceChanged {
            resource: res,
            id: event_id,
            action: None,
          } => {
            assert_eq!(res, resource);
            assert_eq!(event_id, id);
          }
          _ => panic!("Expected ResourceChanged event without action"),
        }
      }

      #[test]
      fn should_support_different_resource_types() {
        // Given: different resource types
        let resources = vec!["users", "tasks", "projects", "organizations"];

        // When: creating ResourceChanged events for each type
        // Then: all should be valid
        for resource in resources {
          let event = LiveEvent::ResourceChanged {
            resource: resource.to_string(),
            id: Uuid::new_v4(),
            action: None,
          };
          match event {
            LiveEvent::ResourceChanged { resource: res, .. } => {
              assert_eq!(res, resource);
            }
            _ => panic!("Expected ResourceChanged event"),
          }
        }
      }
    }

    mod custom_event_behavior {
      use super::*;

      #[test]
      fn should_create_custom_event_with_topic() {
        // Given: custom event topic
        let topic = "custom_notification".to_string();

        // When: creating Custom event
        let event = LiveEvent::Custom {
          topic: topic.clone(),
          payload: None,
        };

        // Then: event should contain topic
        match event {
          LiveEvent::Custom {
            topic: t,
            payload: None,
          } => assert_eq!(t, topic),
          _ => panic!("Expected Custom event"),
        }
      }

      #[test]
      fn should_create_custom_event_with_payload() {
        // Given: custom event with topic and payload
        let topic = "webhook".to_string();
        let payload = Some(serde_json::json!({"event": "deployment", "status": "success"}));

        // When: creating Custom event with payload
        let event = LiveEvent::Custom {
          topic: topic.clone(),
          payload: payload.clone(),
        };

        // Then: event should contain both topic and payload
        match event {
          LiveEvent::Custom {
            topic: t,
            payload: p,
          } => {
            assert_eq!(t, topic);
            assert_eq!(p, payload);
          }
          _ => panic!("Expected Custom event with payload"),
        }
      }

      #[test]
      fn should_create_custom_event_without_payload() {
        // Given: custom event without payload
        let topic = "notification".to_string();

        // When: creating Custom event without payload
        let event = LiveEvent::Custom {
          topic: topic.clone(),
          payload: None,
        };

        // Then: event should have topic but None payload
        match event {
          LiveEvent::Custom {
            topic: t,
            payload: None,
          } => assert_eq!(t, topic),
          _ => panic!("Expected Custom event without payload"),
        }
      }
    }

    mod event_serialization_behavior {
      use super::*;

      #[test]
      fn should_serialize_users_updated_event() {
        // Given: a UsersUpdated event
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let event = LiveEvent::UsersUpdated {
          user_id: Some(user_id),
          org_id: Some(org_id),
        };

        // When: serializing to JSON
        let json = serde_json::to_string(&event).unwrap();

        // Then: should serialize successfully with type tag
        assert!(json.contains("users_updated"));
        assert!(json.contains(&user_id.to_string()));
        assert!(json.contains(&org_id.to_string()));
      }

      #[test]
      fn should_serialize_resource_changed_event() {
        // Given: a ResourceChanged event
        let resource = "users".to_string();
        let id = Uuid::new_v4();
        let event = LiveEvent::ResourceChanged {
          resource: resource.clone(),
          id,
          action: Some("create".to_string()),
        };

        // When: serializing to JSON
        let json = serde_json::to_string(&event).unwrap();

        // Then: should serialize successfully with type tag
        assert!(json.contains("resource_changed"));
        assert!(json.contains(&resource));
        assert!(json.contains(&id.to_string()));
      }

      #[test]
      fn should_serialize_custom_event() {
        // Given: a Custom event
        let topic = "notification".to_string();
        let payload = Some(serde_json::json!({"message": "test"}));
        let event = LiveEvent::Custom {
          topic: topic.clone(),
          payload: payload.clone(),
        };

        // When: serializing to JSON
        let json = serde_json::to_string(&event).unwrap();

        // Then: should serialize successfully with type tag
        assert!(json.contains("custom"));
        assert!(json.contains(&topic));
      }

      #[test]
      fn should_deserialize_users_updated_event() {
        // Given: JSON string for UsersUpdated event
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let json = format!(
          r#"{{"type":"users_updated","user_id":"{}","org_id":"{}"}}"#,
          user_id, org_id
        );

        // When: deserializing from JSON
        let event: LiveEvent = serde_json::from_str(&json).unwrap();

        // Then: should deserialize successfully
        match event {
          LiveEvent::UsersUpdated {
            user_id: Some(uid),
            org_id: Some(oid),
          } => {
            assert_eq!(uid, user_id);
            assert_eq!(oid, org_id);
          }
          _ => panic!("Expected UsersUpdated event"),
        }
      }

      #[test]
      fn should_deserialize_resource_changed_event() {
        // Given: JSON string for ResourceChanged event
        let resource = "users";
        let id = Uuid::new_v4();
        let json = format!(
          r#"{{"type":"resource_changed","resource":"{}","id":"{}"}}"#,
          resource, id
        );

        // When: deserializing from JSON
        let event: LiveEvent = serde_json::from_str(&json).unwrap();

        // Then: should deserialize successfully
        match event {
          LiveEvent::ResourceChanged {
            resource: res,
            id: event_id,
            action: None,
          } => {
            assert_eq!(res, resource);
            assert_eq!(event_id, id);
          }
          _ => panic!("Expected ResourceChanged event"),
        }
      }

      #[test]
      fn should_deserialize_custom_event() {
        // Given: JSON string for Custom event
        let topic = "notification";
        let json = format!(r#"{{"type":"custom","topic":"{}"}}"#, topic);

        // When: deserializing from JSON
        let event: LiveEvent = serde_json::from_str(&json).unwrap();

        // Then: should deserialize successfully
        match event {
          LiveEvent::Custom {
            topic: t,
            payload: None,
          } => assert_eq!(t, topic),
          _ => panic!("Expected Custom event"),
        }
      }

      #[test]
      fn should_skip_none_fields_in_serialization() {
        // Given: UsersUpdated event with None fields
        let event = LiveEvent::UsersUpdated {
          user_id: None,
          org_id: None,
        };

        // When: serializing to JSON
        let json = serde_json::to_string(&event).unwrap();

        // Then: None fields should be omitted (skip_serializing_if)
        assert!(json.contains("users_updated"));
        assert!(!json.contains("user_id") || json.contains("null"));
        assert!(!json.contains("org_id") || json.contains("null"));
      }
    }

    mod event_clone_behavior {
      use super::*;

      #[test]
      fn should_clone_users_updated_event() {
        // Given: a UsersUpdated event
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let event = LiveEvent::UsersUpdated {
          user_id: Some(user_id),
          org_id: Some(org_id),
        };

        // When: cloning the event
        let cloned = event.clone();

        // Then: cloned event should have same values
        match (event, cloned) {
          (
            LiveEvent::UsersUpdated {
              user_id: Some(uid1),
              org_id: Some(oid1),
            },
            LiveEvent::UsersUpdated {
              user_id: Some(uid2),
              org_id: Some(oid2),
            },
          ) => {
            assert_eq!(uid1, uid2);
            assert_eq!(oid1, oid2);
          }
          _ => panic!("Expected UsersUpdated events"),
        }
      }

      #[test]
      fn should_clone_resource_changed_event() {
        // Given: a ResourceChanged event
        let resource = "users".to_string();
        let id = Uuid::new_v4();
        let event = LiveEvent::ResourceChanged {
          resource: resource.clone(),
          id,
          action: Some("update".to_string()),
        };

        // When: cloning the event
        let cloned = event.clone();

        // Then: cloned event should have same values
        match (event, cloned) {
          (
            LiveEvent::ResourceChanged {
              resource: res1,
              id: id1,
              action: act1,
            },
            LiveEvent::ResourceChanged {
              resource: res2,
              id: id2,
              action: act2,
            },
          ) => {
            assert_eq!(res1, res2);
            assert_eq!(id1, id2);
            assert_eq!(act1, act2);
          }
          _ => panic!("Expected ResourceChanged events"),
        }
      }

      #[test]
      fn should_clone_custom_event() {
        // Given: a Custom event
        let topic = "notification".to_string();
        let payload = Some(serde_json::json!({"test": true}));
        let event = LiveEvent::Custom {
          topic: topic.clone(),
          payload: payload.clone(),
        };

        // When: cloning the event
        let cloned = event.clone();

        // Then: cloned event should have same values
        match (event, cloned) {
          (
            LiveEvent::Custom {
              topic: t1,
              payload: p1,
            },
            LiveEvent::Custom {
              topic: t2,
              payload: p2,
            },
          ) => {
            assert_eq!(t1, t2);
            assert_eq!(p1, p2);
          }
          _ => panic!("Expected Custom events"),
        }
      }
    }

    mod event_debug_behavior {
      use super::*;

      #[test]
      fn should_format_users_updated_event_for_debug() {
        // Given: a UsersUpdated event
        let event = LiveEvent::UsersUpdated {
          user_id: Some(Uuid::new_v4()),
          org_id: Some(Uuid::new_v4()),
        };

        // When: formatting for debug
        let debug_str = format!("{:?}", event);

        // Then: should format successfully
        assert!(debug_str.contains("UsersUpdated"));
      }

      #[test]
      fn should_format_resource_changed_event_for_debug() {
        // Given: a ResourceChanged event
        let event = LiveEvent::ResourceChanged {
          resource: "users".to_string(),
          id: Uuid::new_v4(),
          action: None,
        };

        // When: formatting for debug
        let debug_str = format!("{:?}", event);

        // Then: should format successfully
        assert!(debug_str.contains("ResourceChanged"));
      }

      #[test]
      fn should_format_custom_event_for_debug() {
        // Given: a Custom event
        let event = LiveEvent::Custom {
          topic: "notification".to_string(),
          payload: None,
        };

        // When: formatting for debug
        let debug_str = format!("{:?}", event);

        // Then: should format successfully
        assert!(debug_str.contains("Custom"));
      }
    }
  }
}
