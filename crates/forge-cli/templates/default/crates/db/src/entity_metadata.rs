//! Per-entity API metadata: id, actions, display name, filter/sort/response columns.
//! Each model defines its own [EntityMetadata] in the same file as the SeaORM entity (Option A: centralize for DX).

/// Supported CRUD actions for an entity.
pub const ACTIONS: &[&str] = &["create", "read", "update", "delete"];

/// Per-entity metadata: id (snake_case), supported_actions, optional filter/sort/display and response columns.
#[derive(Debug, Clone)]
pub struct EntityMetadata {
  /// Stable identifier (lowercase snake_case).
  pub id: &'static str,
  /// Which actions this entity supports. Subset of ACTIONS.
  pub supported_actions: &'static [&'static str],
  /// Optional; for UI only.
  pub display_name: Option<&'static str>,
  /// Optional; if absent, app derives from model.
  pub allowed_filter_fields: Option<&'static [&'static str]>,
  /// Optional; if absent, app derives from model.
  pub allowed_sort_fields: Option<&'static [&'static str]>,
  /// Optional allow-list of column names to return in list/get.
  pub response_columns_allow: Option<&'static [&'static str]>,
  /// Optional exclude-list; columns never returned (e.g. password_hash).
  pub response_columns_exclude: Option<&'static [&'static str]>,
}
