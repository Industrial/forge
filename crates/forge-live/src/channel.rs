//! Channel names for scoped broadcast.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// A subscription channel (e.g. `org:{id}`, `resource:users:{id}`).
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Channel(String);

impl Channel {
  /// Channel for all connections in an organization.
  pub fn org(org_id: Uuid) -> Self {
    Channel(format!("org:{}", org_id))
  }

  /// Channel for a resource type within an org (permission-scoped live updates).
  /// Format: `org:{org_id}:{resource_type}` (e.g. `org:...:users`, `org:...:roles`).
  pub fn org_resource(org_id: Uuid, resource_type: &str) -> Self {
    Channel(format!("org:{}:{}", org_id, resource_type))
  }

  /// Channel for a specific resource (e.g. `resource:users:{id}`).
  pub fn resource(resource_type: &str, id: Uuid) -> Self {
    Channel(format!("resource:{}:{}", resource_type, id))
  }

  /// Raw channel name (for custom patterns).
  pub fn raw(name: impl Into<String>) -> Self {
    Channel(name.into())
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl fmt::Display for Channel {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}
