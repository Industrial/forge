//! Audit log model (SeaORM) and RestModel impl; read-only (list, get).

use async_trait::async_trait;
use forge_db::DbConnection;
use sea_orm::entity::prelude::*;
use sea_orm::{QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model_error::ModelError;
use crate::query_spec::{FilterCond, FilterOperator, SortDirection, SortSpec};
use crate::rest_model::RestModel;

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

// ---- RestModel (read-only) ----

const FILTER_SORT: &[&str] = &[
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
const SORT_FIELDS: &[&str] = &[
  "id",
  "occurred_at",
  "event_kind",
  "actor_id",
  "action",
  "resource_type",
];

/// REST resource for the audit_log table; implements [RestModel]. Read-only.
pub struct Audit;

#[async_trait]
impl RestModel for Audit {
  type Entity = Entity;
  type Model = Model;

  fn model_id() -> &'static str
  where
    Self: Sized,
  {
    "audit"
  }

  fn filter_fields() -> &'static [&'static str]
  where
    Self: Sized,
  {
    FILTER_SORT
  }

  fn sort_fields() -> &'static [&'static str]
  where
    Self: Sized,
  {
    SORT_FIELDS
  }

  fn response_columns() -> &'static [&'static str]
  where
    Self: Sized,
  {
    FILTER_SORT
  }

  fn display_name() -> Option<&'static str>
  where
    Self: Sized,
  {
    Some("Audit log")
  }

  fn supported_actions() -> &'static [&'static str]
  where
    Self: Sized,
  {
    &["read"]
  }

  fn row_to_json(r: &Self::Model) -> serde_json::Value {
    serde_json::json!({
      "id": r.id.to_string(),
      "event_kind": r.event_kind,
      "actor_id": r.actor_id.to_string(),
      "subject_id": r.subject_id.map(|u| u.to_string()),
      "organization_id": r.organization_id.map(|u| u.to_string()),
      "action": r.action,
      "resource_type": r.resource_type,
      "resource_id": r.resource_id.map(|u| u.to_string()),
      "outcome": r.outcome,
      "reason": r.reason,
      "occurred_at": r.occurred_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
    })
  }

  fn apply_filter(select: sea_orm::Select<Entity>, cond: &FilterCond) -> sea_orm::Select<Entity> {
    apply_filter(select, cond)
  }

  fn default_sort(select: sea_orm::Select<Entity>) -> sea_orm::Select<Entity> {
    select.order_by_desc(Column::OccurredAt)
  }

  fn apply_sort(select: sea_orm::Select<Entity>, sort: &SortSpec) -> sea_orm::Select<Entity> {
    let (col, dir) = match sort.field.as_str() {
      "id" => (Column::Id, sort.direction),
      "occurred_at" => (Column::OccurredAt, sort.direction),
      "event_kind" => (Column::EventKind, sort.direction),
      "actor_id" => (Column::ActorId, sort.direction),
      "action" => (Column::Action, sort.direction),
      "resource_type" => (Column::ResourceType, sort.direction),
      _ => (Column::OccurredAt, sort.direction),
    };
    match dir {
      SortDirection::Asc => select.order_by_asc(col),
      SortDirection::Desc => select.order_by_desc(col),
    }
  }

  async fn create(_db: &DbConnection, _body: serde_json::Value) -> Result<Uuid, ModelError> {
    Err(ModelError::Validation(
      "audit is read-only; create not supported".to_string(),
    ))
  }

  async fn update(
    _db: &DbConnection,
    _id: Uuid,
    _body: serde_json::Value,
  ) -> Result<serde_json::Value, ModelError> {
    Err(ModelError::Validation(
      "audit is read-only; update not supported".to_string(),
    ))
  }

  async fn delete(_db: &DbConnection, _id: Uuid) -> Result<bool, ModelError> {
    Err(ModelError::Validation(
      "audit is read-only; delete not supported".to_string(),
    ))
  }
}

fn apply_filter(select: sea_orm::Select<Entity>, cond: &FilterCond) -> sea_orm::Select<Entity> {
  use sea_orm::ColumnTrait;
  match cond.field.as_str() {
    "id" | "actor_id" => {
      let parse_uuid = |j: &serde_json::Value| j.as_str().and_then(|s| Uuid::parse_str(s).ok());
      let col = if cond.field == "id" {
        Column::Id
      } else {
        Column::ActorId
      };
      match cond.operator {
        FilterOperator::Eq => {
          if let Some(v) = cond.value.as_ref().and_then(parse_uuid) {
            select.filter(col.eq(v))
          } else {
            select
          }
        }
        FilterOperator::Ne => {
          if let Some(v) = cond.value.as_ref().and_then(parse_uuid) {
            select.filter(col.ne(v))
          } else {
            select
          }
        }
        FilterOperator::In => {
          if let Some(serde_json::Value::Array(arr)) = cond.value.as_ref() {
            let vals: Vec<Uuid> = arr.iter().filter_map(parse_uuid).collect();
            if vals.is_empty() {
              select.filter(col.eq(Uuid::nil()))
            } else {
              select.filter(col.is_in(vals))
            }
          } else {
            select
          }
        }
        FilterOperator::IsNull => select.filter(col.is_null()),
        _ => select,
      }
    }
    "subject_id" | "organization_id" | "resource_id" => {
      let parse_uuid = |j: &serde_json::Value| j.as_str().and_then(|s| Uuid::parse_str(s).ok());
      let col = match cond.field.as_str() {
        "subject_id" => Column::SubjectId,
        "organization_id" => Column::OrganizationId,
        _ => Column::ResourceId,
      };
      match cond.operator {
        FilterOperator::Eq => {
          if let Some(v) = cond.value.as_ref().and_then(parse_uuid) {
            select.filter(col.eq(Some(v)))
          } else {
            select
          }
        }
        FilterOperator::Ne => {
          if let Some(v) = cond.value.as_ref().and_then(parse_uuid) {
            select.filter(col.ne(Some(v)))
          } else {
            select
          }
        }
        FilterOperator::In => {
          if let Some(serde_json::Value::Array(arr)) = cond.value.as_ref() {
            let vals: Vec<Uuid> = arr.iter().filter_map(parse_uuid).collect();
            if vals.is_empty() {
              select.filter(col.eq(Some(Uuid::nil())))
            } else {
              select.filter(col.is_in(vals.into_iter().map(Some)))
            }
          } else {
            select
          }
        }
        FilterOperator::IsNull => select.filter(col.is_null()),
        _ => select,
      }
    }
    "event_kind" | "action" | "resource_type" | "outcome" | "reason" => {
      let col = match cond.field.as_str() {
        "event_kind" => Column::EventKind,
        "action" => Column::Action,
        "resource_type" => Column::ResourceType,
        "outcome" => Column::Outcome,
        _ => Column::Reason,
      };
      match cond.operator {
        FilterOperator::Eq => {
          if let Some(serde_json::Value::String(s)) = cond.value.as_ref() {
            select.filter(col.eq(s.as_str()))
          } else {
            select
          }
        }
        FilterOperator::Ne => {
          if let Some(serde_json::Value::String(s)) = cond.value.as_ref() {
            select.filter(col.ne(s.as_str()))
          } else {
            select
          }
        }
        FilterOperator::Contains => {
          if let Some(serde_json::Value::String(s)) = cond.value.as_ref() {
            select.filter(col.contains(s))
          } else {
            select
          }
        }
        FilterOperator::In => {
          if let Some(serde_json::Value::Array(arr)) = cond.value.as_ref() {
            let strs: Vec<&str> = arr.iter().filter_map(|j| j.as_str()).collect();
            if strs.is_empty() {
              select.filter(col.eq(""))
            } else {
              select.filter(col.is_in(strs))
            }
          } else {
            select
          }
        }
        FilterOperator::IsNull => select.filter(col.is_null()),
        _ => select,
      }
    }
    "occurred_at" => {
      let parse_dt = |j: &serde_json::Value| {
        let s = j.as_str()?;
        let s_trim = s.trim_end_matches('Z');
        chrono::NaiveDateTime::parse_from_str(s_trim, "%Y-%m-%dT%H:%M:%S%.f")
          .ok()
          .or_else(|| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
      };
      match cond.operator {
        FilterOperator::Eq => {
          if let Some(v) = cond.value.as_ref().and_then(parse_dt) {
            select.filter(Column::OccurredAt.eq(v))
          } else {
            select
          }
        }
        FilterOperator::Ne => {
          if let Some(v) = cond.value.as_ref().and_then(parse_dt) {
            select.filter(Column::OccurredAt.ne(v))
          } else {
            select
          }
        }
        FilterOperator::IsNull => select.filter(Column::OccurredAt.is_null()),
        _ => select,
      }
    }
    _ => select,
  }
}
