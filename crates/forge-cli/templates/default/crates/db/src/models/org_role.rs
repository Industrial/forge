//! Org role model (SeaORM) and RestModel impl for the generic REST handler.

use async_trait::async_trait;
use forge_db::DbConnection;
use sea_orm::entity::prelude::*;
use sea_orm::{QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model_error::ModelError;
use crate::query_spec::{FilterCond, FilterOperator, SortDirection, SortSpec};
use crate::rest_model::{REST_ACTIONS, RestModel};

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

// ---- RestModel ----

const FILTER_SORT_RESPONSE: &[&str] = &[
  "id",
  "org_id",
  "name",
  "display_name",
  "created_at",
  "updated_at",
];

/// REST resource for the org_role table; implements [RestModel].
pub struct Role;

#[async_trait]
impl RestModel for Role {
  type Entity = Entity;
  type Model = Model;

  fn model_id() -> &'static str
  where
    Self: Sized,
  {
    "role"
  }

  fn filter_fields() -> &'static [&'static str]
  where
    Self: Sized,
  {
    FILTER_SORT_RESPONSE
  }

  fn sort_fields() -> &'static [&'static str]
  where
    Self: Sized,
  {
    FILTER_SORT_RESPONSE
  }

  fn response_columns() -> &'static [&'static str]
  where
    Self: Sized,
  {
    FILTER_SORT_RESPONSE
  }

  fn display_name() -> Option<&'static str>
  where
    Self: Sized,
  {
    Some("Role")
  }

  fn supported_actions() -> &'static [&'static str]
  where
    Self: Sized,
  {
    REST_ACTIONS
  }

  fn row_to_json(r: &Self::Model) -> serde_json::Value {
    serde_json::json!({
      "id": r.id.to_string(),
      "org_id": r.org_id.to_string(),
      "name": r.name,
      "display_name": r.display_name,
      "created_at": r.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
      "updated_at": r.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
    })
  }

  fn apply_filter(select: sea_orm::Select<Entity>, cond: &FilterCond) -> sea_orm::Select<Entity> {
    apply_filter(select, cond)
  }

  fn default_sort(select: sea_orm::Select<Entity>) -> sea_orm::Select<Entity> {
    select.order_by_asc(Column::Name)
  }

  fn apply_sort(select: sea_orm::Select<Entity>, sort: &SortSpec) -> sea_orm::Select<Entity> {
    let (col, dir) = match sort.field.as_str() {
      "id" => (Column::Id, sort.direction),
      "org_id" => (Column::OrgId, sort.direction),
      "name" => (Column::Name, sort.direction),
      "display_name" => (Column::DisplayName, sort.direction),
      "created_at" => (Column::CreatedAt, sort.direction),
      "updated_at" => (Column::UpdatedAt, sort.direction),
      _ => (Column::Name, sort.direction),
    };
    match dir {
      SortDirection::Asc => select.order_by_asc(col),
      SortDirection::Desc => select.order_by_desc(col),
    }
  }

  async fn create(_db: &DbConnection, _body: serde_json::Value) -> Result<Uuid, ModelError> {
    Err(ModelError::Validation(
      "role create not implemented via generic handler; use POST /api/organizations/:id/roles"
        .to_string(),
    ))
  }

  async fn update(
    _db: &DbConnection,
    _id: Uuid,
    _body: serde_json::Value,
  ) -> Result<serde_json::Value, ModelError> {
    Err(ModelError::Validation(
      "role update not implemented via generic handler; use PATCH /api/organizations/:id/roles/:role_id".to_string(),
    ))
  }

  async fn delete(_db: &DbConnection, _id: Uuid) -> Result<bool, ModelError> {
    Err(ModelError::Validation(
      "role delete not implemented via generic handler; use DELETE /api/organizations/:id/roles/:role_id".to_string(),
    ))
  }
}

fn apply_filter(select: sea_orm::Select<Entity>, cond: &FilterCond) -> sea_orm::Select<Entity> {
  use sea_orm::ColumnTrait;
  match cond.field.as_str() {
    "id" | "org_id" => {
      let parse_uuid = |j: &serde_json::Value| j.as_str().and_then(|s| Uuid::parse_str(s).ok());
      let col = if cond.field == "id" {
        Column::Id
      } else {
        Column::OrgId
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
    "name" | "display_name" => {
      let col = if cond.field == "name" {
        Column::Name
      } else {
        Column::DisplayName
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
        FilterOperator::StartsWith => {
          if let Some(serde_json::Value::String(s)) = cond.value.as_ref() {
            select.filter(col.starts_with(s))
          } else {
            select
          }
        }
        FilterOperator::EndsWith => {
          if let Some(serde_json::Value::String(s)) = cond.value.as_ref() {
            select.filter(col.ends_with(s))
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
    "created_at" | "updated_at" => {
      let col = if cond.field == "created_at" {
        Column::CreatedAt
      } else {
        Column::UpdatedAt
      };
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
            select.filter(col.eq(v))
          } else {
            select
          }
        }
        FilterOperator::Ne => {
          if let Some(v) = cond.value.as_ref().and_then(parse_dt) {
            select.filter(col.ne(v))
          } else {
            select
          }
        }
        FilterOperator::IsNull => select.filter(col.is_null()),
        _ => select,
      }
    }
    _ => select,
  }
}
