use crate::entity_metadata::{EntityMetadata, ACTIONS};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Allowed filter/sort/response columns. Defined alongside the model (Option A).
const FILTER_SORT_RESPONSE_COLUMNS: &[&str] =
  &["id", "org_id", "name", "display_name", "created_at", "updated_at"];

/// Entity metadata: API id "role", actions, display name, filter/sort/response columns. Defined alongside the model (Option A).
pub const ENTITY_METADATA: EntityMetadata = EntityMetadata {
  id: "role",
  supported_actions: ACTIONS,
  display_name: Some("Role"),
  allowed_filter_fields: Some(FILTER_SORT_RESPONSE_COLUMNS),
  allowed_sort_fields: Some(FILTER_SORT_RESPONSE_COLUMNS),
  response_columns_allow: Some(FILTER_SORT_RESPONSE_COLUMNS),
  response_columns_exclude: None,
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "org_role")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  pub org_id: Uuid,
  pub name: String,
  pub display_name: Option<String>,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
