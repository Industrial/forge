//! User model (SeaORM) and RestModel impl for the generic REST handler. Response columns exclude password_hash.

use async_trait::async_trait;
use forge_auth::AuthzContext;
use forge_db::DbConnection;
use sea_orm::entity::prelude::*;
use sea_orm::{QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model_error::ModelError;
use crate::query_spec::{FilterCond, FilterOperator, SortDirection, SortSpec};
use crate::rest_model::{REST_ACTIONS, RestModel};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  #[sea_orm(unique)]
  pub email: String,
  pub password_hash: String,
  pub is_active: bool,
  pub is_admin: bool,
  /// Deprecated for authz: session profile (current_org_id, current_role_name) is the source of truth. Kept for seeds/display.
  pub current_org_id: Option<Uuid>,
  /// Deprecated for authz: use session profile. Kept for seeds/display.
  pub current_role: Option<String>,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

impl axum_login::AuthUser for Model {
  type Id = Uuid;
  fn id(&self) -> Self::Id {
    self.id
  }
  fn session_auth_hash(&self) -> &[u8] {
    self.password_hash.as_bytes()
  }
}

impl AuthzContext for Model {
  type RequesterId = Uuid;
  type SubjectId = Uuid;

  fn requester_id(&self) -> Uuid {
    self.id
  }
  fn subject_id(&self) -> Uuid {
    self.id
  }
  fn organization_id(&self) -> Option<Uuid> {
    self.current_org_id
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// ---- RestModel ----

const FILTER_SORT: &[&str] = &["id", "email", "is_active", "created_at", "updated_at"];
const RESPONSE_COLUMNS: &[&str] = &["id", "email", "is_active", "created_at", "updated_at"];

/// REST resource for the user table; implements [RestModel]. Excludes password_hash from responses.
pub struct User;

#[async_trait]
impl RestModel for User {
  type Entity = Entity;
  type Model = Model;

  fn model_id() -> &'static str
  where
    Self: Sized,
  {
    "user"
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
    FILTER_SORT
  }

  fn response_columns() -> &'static [&'static str]
  where
    Self: Sized,
  {
    RESPONSE_COLUMNS
  }

  fn display_name() -> Option<&'static str>
  where
    Self: Sized,
  {
    Some("User")
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
      "email": r.email,
      "is_active": r.is_active,
      "created_at": r.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
      "updated_at": r.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
    })
  }

  fn apply_filter(select: sea_orm::Select<Entity>, cond: &FilterCond) -> sea_orm::Select<Entity> {
    apply_filter(select, cond)
  }

  fn default_sort(select: sea_orm::Select<Entity>) -> sea_orm::Select<Entity> {
    select.order_by_asc(Column::Email)
  }

  fn apply_sort(select: sea_orm::Select<Entity>, sort: &SortSpec) -> sea_orm::Select<Entity> {
    let (col, dir) = match sort.field.as_str() {
      "id" => (Column::Id, sort.direction),
      "email" => (Column::Email, sort.direction),
      "is_active" => (Column::IsActive, sort.direction),
      "created_at" => (Column::CreatedAt, sort.direction),
      "updated_at" => (Column::UpdatedAt, sort.direction),
      _ => (Column::Email, sort.direction),
    };
    match dir {
      SortDirection::Asc => select.order_by_asc(col),
      SortDirection::Desc => select.order_by_desc(col),
    }
  }

  async fn create(_db: &DbConnection, _body: serde_json::Value) -> Result<Uuid, ModelError> {
    Err(ModelError::Validation(
      "user create not implemented via generic handler; use POST /api/users".to_string(),
    ))
  }

  async fn update(
    _db: &DbConnection,
    _id: Uuid,
    _body: serde_json::Value,
  ) -> Result<serde_json::Value, ModelError> {
    Err(ModelError::Validation(
      "user update not implemented via generic handler; use PATCH /api/users/:id".to_string(),
    ))
  }

  async fn delete(_db: &DbConnection, _id: Uuid) -> Result<bool, ModelError> {
    Err(ModelError::Validation(
      "user delete not implemented via generic handler; use DELETE /api/users/:id".to_string(),
    ))
  }
}

fn apply_filter(select: sea_orm::Select<Entity>, cond: &FilterCond) -> sea_orm::Select<Entity> {
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
    "email" => match cond.operator {
      FilterOperator::Eq => {
        if let Some(serde_json::Value::String(s)) = cond.value.as_ref() {
          select.filter(Column::Email.eq(s.as_str()))
        } else {
          select
        }
      }
      FilterOperator::Ne => {
        if let Some(serde_json::Value::String(s)) = cond.value.as_ref() {
          select.filter(Column::Email.ne(s.as_str()))
        } else {
          select
        }
      }
      FilterOperator::Contains => {
        if let Some(serde_json::Value::String(s)) = cond.value.as_ref() {
          select.filter(Column::Email.contains(s))
        } else {
          select
        }
      }
      FilterOperator::StartsWith | FilterOperator::EndsWith => {
        if let Some(serde_json::Value::String(s)) = cond.value.as_ref() {
          let sel = if cond.operator == FilterOperator::StartsWith {
            select.filter(Column::Email.starts_with(s))
          } else {
            select.filter(Column::Email.ends_with(s))
          };
          sel
        } else {
          select
        }
      }
      FilterOperator::In => {
        if let Some(serde_json::Value::Array(arr)) = cond.value.as_ref() {
          let strs: Vec<&str> = arr.iter().filter_map(|j| j.as_str()).collect();
          if strs.is_empty() {
            select.filter(Column::Email.eq(""))
          } else {
            select.filter(Column::Email.is_in(strs))
          }
        } else {
          select
        }
      }
      FilterOperator::IsNull => select.filter(Column::Email.is_null()),
      _ => select,
    },
    "is_active" => {
      let parse_bool = |j: &serde_json::Value| j.as_bool();
      match cond.operator {
        FilterOperator::Eq => {
          if let Some(v) = cond.value.as_ref().and_then(parse_bool) {
            select.filter(Column::IsActive.eq(v))
          } else {
            select
          }
        }
        FilterOperator::Ne => {
          if let Some(v) = cond.value.as_ref().and_then(parse_bool) {
            select.filter(Column::IsActive.ne(v))
          } else {
            select
          }
        }
        FilterOperator::IsNull => select.filter(Column::IsActive.is_null()),
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
