use crate::entity_metadata::{EntityMetadata, ACTIONS};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Allowed filter/sort/response columns. Defined alongside the model (Option A).
const FILTER_SORT_RESPONSE_COLUMNS: &[&str] =
  &["id", "scope", "role_name", "permission_key", "org_id"];

/// Entity metadata: API id "permission", actions, display name, filter/sort/response columns. Defined alongside the model (Option A).
pub const ENTITY_METADATA: EntityMetadata = EntityMetadata {
  id: "permission",
  supported_actions: ACTIONS,
  display_name: Some("Permission"),
  allowed_filter_fields: Some(FILTER_SORT_RESPONSE_COLUMNS),
  allowed_sort_fields: Some(FILTER_SORT_RESPONSE_COLUMNS),
  response_columns_allow: Some(FILTER_SORT_RESPONSE_COLUMNS),
  response_columns_exclude: None,
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "role_permission")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  pub scope: String,
  pub role_name: String,
  pub permission_key: String,
  /// Null for global scope (platform_admin); set for org-scope rows.
  pub org_id: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
