//! Role permission model (SeaORM) and RestModel impl for the generic REST handler.

use async_trait::async_trait;
use forge_db::DbConnection;
use sea_orm::entity::prelude::*;
use sea_orm::{QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model_error::ModelError;
use crate::query_spec::{FilterCond, FilterOperator, SortDirection, SortSpec};
use crate::rest_model::{RestModel, REST_ACTIONS};

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

// ---- RestModel ----

const FILTER_SORT_RESPONSE: &[&str] =
  &["id", "scope", "role_name", "permission_key", "org_id"];

/// REST resource for the role_permission table; implements [RestModel].
pub struct Permission;

#[async_trait]
impl RestModel for Permission {
  type Entity = Entity;
  type Model = Model;

  fn model_id() -> &'static str
  where
    Self: Sized,
  {
    "permission"
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
    Some("Permission")
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
      "scope": r.scope,
      "role_name": r.role_name,
      "permission_key": r.permission_key,
      "org_id": r.org_id.map(|u| u.to_string()),
    })
  }

  fn apply_filter(
    select: sea_orm::Select<Entity>,
    cond: &FilterCond,
  ) -> sea_orm::Select<Entity> {
    apply_filter(select, cond)
  }

  fn default_sort(select: sea_orm::Select<Entity>) -> sea_orm::Select<Entity> {
    select.order_by_asc(Column::RoleName)
  }

  fn apply_sort(
    select: sea_orm::Select<Entity>,
    sort: &SortSpec,
  ) -> sea_orm::Select<Entity> {
    let (col, dir) = match sort.field.as_str() {
      "id" => (Column::Id, sort.direction),
      "scope" => (Column::Scope, sort.direction),
      "role_name" => (Column::RoleName, sort.direction),
      "permission_key" => (Column::PermissionKey, sort.direction),
      "org_id" => (Column::OrgId, sort.direction),
      _ => (Column::RoleName, sort.direction),
    };
    match dir {
      SortDirection::Asc => select.order_by_asc(col),
      SortDirection::Desc => select.order_by_desc(col),
    }
  }

  async fn create(
    _db: &DbConnection,
    _body: serde_json::Value,
  ) -> Result<Uuid, ModelError> {
    Err(ModelError::Validation(
      "permission create not implemented via generic handler".to_string(),
    ))
  }

  async fn update(
    _db: &DbConnection,
    _id: Uuid,
    _body: serde_json::Value,
  ) -> Result<serde_json::Value, ModelError> {
    Err(ModelError::Validation(
      "permission update not implemented via generic handler".to_string(),
    ))
  }

  async fn delete(_db: &DbConnection, _id: Uuid) -> Result<bool, ModelError> {
    Err(ModelError::Validation(
      "permission delete not implemented via generic handler".to_string(),
    ))
  }
}

fn apply_filter(
  select: sea_orm::Select<Entity>,
  cond: &FilterCond,
) -> sea_orm::Select<Entity> {
  use sea_orm::ColumnTrait;
  match cond.field.as_str() {
    "id" => {
      let parse_uuid = |j: &serde_json::Value| j.as_str().and_then(|s| Uuid::parse_str(s).ok());
      match cond.operator {
        FilterOperator::Eq => {
          if let Some(v) = cond.value.as_ref().and_then(parse_uuid) {
            select.filter(Column::Id.eq(v))
          } else {
            select
          }
        }
        FilterOperator::Ne => {
          if let Some(v) = cond.value.as_ref().and_then(parse_uuid) {
            select.filter(Column::Id.ne(v))
          } else {
            select
          }
        }
        FilterOperator::In => {
          if let Some(serde_json::Value::Array(arr)) = cond.value.as_ref() {
            let vals: Vec<Uuid> = arr.iter().filter_map(parse_uuid).collect();
            if vals.is_empty() {
              select.filter(Column::Id.eq(Uuid::nil()))
            } else {
              select.filter(Column::Id.is_in(vals))
            }
          } else {
            select
          }
        }
        FilterOperator::IsNull => select.filter(Column::Id.is_null()),
        _ => select,
      }
    }
    "org_id" => {
      let parse_uuid = |j: &serde_json::Value| j.as_str().and_then(|s| Uuid::parse_str(s).ok());
      match cond.operator {
        FilterOperator::Eq => {
          if let Some(v) = cond.value.as_ref().and_then(parse_uuid) {
            select.filter(Column::OrgId.eq(Some(v)))
          } else {
            select
          }
        }
        FilterOperator::Ne => {
          if let Some(v) = cond.value.as_ref().and_then(parse_uuid) {
            select.filter(Column::OrgId.ne(Some(v)))
          } else {
            select
          }
        }
        FilterOperator::In => {
          if let Some(serde_json::Value::Array(arr)) = cond.value.as_ref() {
            let vals: Vec<Uuid> = arr.iter().filter_map(parse_uuid).collect();
            if vals.is_empty() {
              select.filter(Column::OrgId.eq(Some(Uuid::nil())))
            } else {
              select.filter(Column::OrgId.is_in(vals.into_iter().map(Some)))
            }
          } else {
            select
          }
        }
        FilterOperator::IsNull => select.filter(Column::OrgId.is_null()),
        _ => select,
      }
    }
    "scope" | "role_name" | "permission_key" => {
      let col = match cond.field.as_str() {
        "scope" => Column::Scope,
        "role_name" => Column::RoleName,
        _ => Column::PermissionKey,
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
    _ => select,
  }
}
