//! Entity registry per docs/technical-choices/03-entity-registry.md.
//! Metadata (including filter/sort/response columns) is defined in each db model file (Option A); this module re-exports and adds helpers.

use std::sync::OnceLock;

pub use db::entity_metadata::{EntityMetadata, ACTIONS};

const EMPTY_FIELDS: &[&'static str] = &[];

/// Returns the full registry: one entry per entity. Sourced from db model files.
#[inline]
pub fn registry() -> &'static [EntityMetadata] {
  db::registry::entity_registry()
}

/// Returns the entity with the given id, or None if unknown (§8).
#[inline]
pub fn get_entity(id: &str) -> Option<&'static EntityMetadata> {
  registry().iter().find(|e| e.id == id)
}

/// Returns whether the given id is a known entity.
#[inline]
pub fn is_known_entity(id: &str) -> bool {
  get_entity(id).is_some()
}

/// Returns the allowed filter fields for an entity. From metadata in the entity file; empty if unset.
#[inline]
pub fn effective_filter_fields(entity_id: &str) -> &'static [&'static str] {
  get_entity(entity_id)
    .and_then(|e| e.allowed_filter_fields)
    .unwrap_or(EMPTY_FIELDS)
}

/// Returns the allowed sort fields for an entity. From metadata in the entity file; empty if unset.
#[inline]
pub fn effective_sort_fields(entity_id: &str) -> &'static [&'static str] {
  get_entity(entity_id)
    .and_then(|e| e.allowed_sort_fields)
    .unwrap_or(EMPTY_FIELDS)
}

/// Returns the column names to include in list/get responses. From metadata in the entity file (response_columns_allow, or allow-list minus response_columns_exclude).
pub fn effective_response_columns(entity_id: &str) -> Vec<&'static str> {
  let meta = match get_entity(entity_id) {
    Some(m) => m,
    None => return vec![],
  };
  if let Some(allow) = meta.response_columns_allow {
    return allow.to_vec();
  }
  // No allow-list: nothing to return (entity file should set response_columns_allow or response_columns_exclude and a default is not defined here).
  vec![]
}

static ALLOWED_KEYS: OnceLock<Vec<&'static str>> = OnceLock::new();

/// Allowed permission keys derived from the registry (§6): for each entity, `<id>.<action>` for each
/// supported action, plus `all.read` and `all.write`. Computed once at first use.
/// Keys are derived from the registry so adding a new entity requires no code change here.
pub fn allowed_permission_keys() -> &'static [&'static str] {
  ALLOWED_KEYS.get_or_init(|| {
    let mut keys: Vec<&'static str> = registry()
      .iter()
      .flat_map(|e| {
        e.supported_actions.iter().map(move |&action| {
          let s = format!("{}.{}", e.id, action);
          let leaked = Box::leak(s.into_boxed_str());
          &*leaked
        })
      })
      .collect();
    keys.push("all.read");
    keys.push("all.write");
    keys
  })
}
