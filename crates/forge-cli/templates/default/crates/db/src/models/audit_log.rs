use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

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
