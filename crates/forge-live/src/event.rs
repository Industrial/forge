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
