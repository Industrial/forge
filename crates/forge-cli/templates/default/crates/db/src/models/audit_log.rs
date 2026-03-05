use crate::entity_metadata::EntityMetadata;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Allowed filter/sort columns. Defined alongside the model (Option A).
const FILTER_SORT_COLUMNS: &[&str] = &[
  "id",
  "event_kind",
  "actor_id",
  "subject_id",
  "organization_id",
  "action",
  "resource_type",
  "resource_id",
  "outcome",
  "reason",
  "occurred_at",
];
/// Allowed sort fields (subset with common ordering).
const SORT_COLUMNS: &[&str] = &["id", "occurred_at", "event_kind", "actor_id", "action", "resource_type"];

/// Entity metadata: API id "audit", read-only, filter/sort/response columns. Defined alongside the model (Option A).
pub const ENTITY_METADATA: EntityMetadata = EntityMetadata {
  id: "audit",
  supported_actions: &["read"],
  display_name: Some("Audit log"),
  allowed_filter_fields: Some(FILTER_SORT_COLUMNS),
  allowed_sort_fields: Some(SORT_COLUMNS),
  response_columns_allow: Some(FILTER_SORT_COLUMNS),
  response_columns_exclude: None,
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "audit_log")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  pub event_kind: String,
  pub actor_id: Uuid,
  pub subject_id: Option<Uuid>,
  pub organization_id: Option<Uuid>,
  pub action: String,
  pub resource_type: String,
  pub resource_id: Option<Uuid>,
  pub outcome: String,
  pub reason: Option<String>,
  pub occurred_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
