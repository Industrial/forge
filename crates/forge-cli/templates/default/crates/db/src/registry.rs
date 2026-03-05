//! Aggregates per-entity metadata from model files into a single registry.

use crate::entity_metadata::EntityMetadata;
use crate::models;

/// Returns the full entity registry: one entry per API entity (organization, user, role, permission, audit).
#[inline]
pub fn entity_registry() -> &'static [EntityMetadata] {
  &[
    models::organization::ENTITY_METADATA,
    models::user::ENTITY_METADATA,
    models::org_role::ENTITY_METADATA,
    models::role_permission::ENTITY_METADATA,
    models::audit_log::ENTITY_METADATA,
  ]
}
