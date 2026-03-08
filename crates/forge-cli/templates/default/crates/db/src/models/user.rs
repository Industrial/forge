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
      "create not supported for this model; use POST /api/dashboard/users".to_string(),
    ))
  }

  async fn update(
    _db: &DbConnection,
    _id: Uuid,
    _body: serde_json::Value,
  ) -> Result<serde_json::Value, ModelError> {
    Err(ModelError::Validation(
      "update not supported for this model; use PATCH /api/dashboard/users/:id".to_string(),
    ))
  }

  async fn delete(_db: &DbConnection, _id: Uuid) -> Result<bool, ModelError> {
    Err(ModelError::Validation(
      "delete not supported for this model; use DELETE /api/dashboard/users/:id".to_string(),
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
          if cond.operator == FilterOperator::StartsWith {
            select.filter(Column::Email.starts_with(s))
          } else {
            select.filter(Column::Email.ends_with(s))
          }
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

#[cfg(test)]
mod bdd_tests {
  use super::*;
  use chrono::Utc;
  use sea_orm::{Database, EntityTrait, Set};
  use sea_orm_migration::MigratorTrait;
  use uuid::Uuid;

  async fn test_db() -> forge_db::DbConnection {
    let conn = Database::connect(sea_orm::ConnectOptions::new("sqlite::memory:".to_string()))
      .await
      .unwrap();
    migrations::Migrator::up(&conn, None)
      .await
      .expect("migrate");
    forge_db::wrap_traced(conn)
  }

  mod rest_model_behavior {
    use super::*;

    #[test]
    fn should_have_model_id_user() {
      // Given: User RestModel implementation
      // When: checking model_id
      // Then: should return "user"
      assert_eq!(User::model_id(), "user");
    }

    #[test]
    fn should_have_display_name() {
      // Given: User RestModel implementation
      // When: checking display_name
      // Then: should return Some("User")
      assert_eq!(User::display_name(), Some("User"));
    }

    #[test]
    fn should_exclude_password_hash_from_response_columns() {
      // Given: User RestModel implementation
      // When: checking response_columns
      // Then: should not include password_hash
      let cols = User::response_columns();
      assert!(!cols.contains(&"password_hash"));
      assert!(cols.contains(&"id"));
      assert!(cols.contains(&"email"));
    }

    #[tokio::test]
    async fn should_reject_create_via_generic_handler() {
      // Given: User RestModel and a test database
      let db = test_db().await;
      let body = serde_json::json!({});

      // When: attempting to create a user via generic handler
      let result = User::create(&db, body).await;

      // Then: should return Validation error
      assert!(result.is_err());
      match result.unwrap_err() {
        ModelError::Validation(msg) => {
          assert!(msg.contains("not supported"));
        }
        _ => panic!("Expected Validation error"),
      }
    }

    #[tokio::test]
    async fn should_reject_update_via_generic_handler() {
      // Given: User RestModel and a test database
      let db = test_db().await;
      let id = Uuid::new_v4();
      let body = serde_json::json!({});

      // When: attempting to update a user via generic handler
      let result = User::update(&db, id, body).await;

      // Then: should return Validation error
      assert!(result.is_err());
      match result.unwrap_err() {
        ModelError::Validation(msg) => {
          assert!(msg.contains("not implemented via generic handler"));
          assert!(msg.contains("PATCH /api/users"));
        }
        _ => panic!("Expected Validation error"),
      }
    }

    #[tokio::test]
    async fn should_reject_delete_via_generic_handler() {
      // Given: User RestModel and a test database
      let db = test_db().await;
      let id = Uuid::new_v4();

      // When: attempting to delete a user via generic handler
      let result = User::delete(&db, id).await;

      // Then: should return Validation error
      assert!(result.is_err());
      match result.unwrap_err() {
        ModelError::Validation(msg) => {
          assert!(msg.contains("not implemented via generic handler"));
          assert!(msg.contains("DELETE /api/users"));
        }
        _ => panic!("Expected Validation error"),
      }
    }
  }

  mod model_structure_behavior {
    use super::*;

    #[tokio::test]
    async fn should_create_user_with_required_fields() {
      // Given: a test database
      let db = test_db().await;
      let user_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();

      // When: inserting a user with required fields
      let result = Entity::insert(ActiveModel {
        id: Set(user_id),
        email: Set("test@example.com".to_string()),
        password_hash: Set("hashed-password".to_string()),
        is_active: Set(true),
        is_admin: Set(false),
        current_org_id: Set(None),
        current_role: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await;

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_store_user_with_email_and_password_hash() {
      // Given: a test database
      let db = test_db().await;
      let user_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();

      // When: inserting a user
      Entity::insert(ActiveModel {
        id: Set(user_id),
        email: Set("user@example.com".to_string()),
        password_hash: Set("hashed-password-123".to_string()),
        is_active: Set(true),
        is_admin: Set(false),
        current_org_id: Set(None),
        current_role: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert");

      // Then: should be able to retrieve it
      let user = Entity::find_by_id(user_id)
        .one(&db)
        .await
        .expect("find")
        .expect("user should exist");
      assert_eq!(user.id, user_id);
      assert_eq!(user.email, "user@example.com");
      assert_eq!(user.password_hash, "hashed-password-123");
    }

    #[tokio::test]
    async fn should_query_user_by_email() {
      // Given: a test database with users
      let db = test_db().await;
      let now = Utc::now().naive_utc();

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set("user1@example.com".to_string()),
        password_hash: Set("hash1".to_string()),
        is_active: Set(true),
        is_admin: Set(false),
        current_org_id: Set(None),
        current_role: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert user1");

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set("user2@example.com".to_string()),
        password_hash: Set("hash2".to_string()),
        is_active: Set(true),
        is_admin: Set(false),
        current_org_id: Set(None),
        current_role: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert user2");

      // When: querying by email
      let user = Entity::find()
        .filter(Column::Email.eq("user1@example.com"))
        .one(&db)
        .await
        .expect("query");

      // Then: should find the correct user
      assert!(user.is_some());
      let u = user.unwrap();
      assert_eq!(u.email, "user1@example.com");
    }
  }

  mod authz_context_behavior {
    use super::*;

    #[tokio::test]
    async fn should_implement_authz_context() {
      // Given: a user model
      let db = test_db().await;
      let user_id = Uuid::new_v4();
      let org_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();

      Entity::insert(ActiveModel {
        id: Set(user_id),
        email: Set("authz@example.com".to_string()),
        password_hash: Set("hash".to_string()),
        is_active: Set(true),
        is_admin: Set(false),
        current_org_id: Set(Some(org_id)),
        current_role: Set(Some("viewer".to_string())),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert");

      let user = Entity::find_by_id(user_id)
        .one(&db)
        .await
        .expect("find")
        .expect("user should exist");

      // When: using AuthzContext methods
      // Then: should return correct values
      assert_eq!(user.requester_id(), user_id);
      assert_eq!(user.subject_id(), user_id);
      assert_eq!(user.organization_id(), Some(org_id));
    }
  }

  mod row_to_json_behavior {
    use super::*;

    #[test]
    fn should_exclude_password_hash_from_json() {
      // Given: a user model with password_hash
      let id = Uuid::new_v4();
      let now = Utc::now().naive_utc();
      let model = Model {
        id,
        email: "test@example.com".to_string(),
        password_hash: "secret-hash".to_string(),
        is_active: true,
        is_admin: false,
        current_org_id: None,
        current_role: None,
        created_at: now,
        updated_at: now,
      };

      // When: converting to JSON
      let json = User::row_to_json(&model);

      // Then: should not include password_hash
      assert!(json.get("password_hash").is_none());
      assert_eq!(json["id"].as_str().unwrap(), id.to_string());
      assert_eq!(json["email"].as_str().unwrap(), "test@example.com");
      assert!(json["is_active"].as_bool().unwrap());
    }
  }
}
