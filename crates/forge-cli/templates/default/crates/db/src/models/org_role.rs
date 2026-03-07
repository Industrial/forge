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
    fn should_have_model_id_role() {
      // Given: Role RestModel implementation
      // When: checking model_id
      // Then: should return "role"
      assert_eq!(Role::model_id(), "role");
    }

    #[test]
    fn should_have_display_name() {
      // Given: Role RestModel implementation
      // When: checking display_name
      // Then: should return Some("Role")
      assert_eq!(Role::display_name(), Some("Role"));
    }

    #[tokio::test]
    async fn should_reject_create_via_generic_handler() {
      // Given: Role RestModel and a test database
      let db = test_db().await;
      let body = serde_json::json!({});

      // When: attempting to create a role via generic handler
      let result = Role::create(&db, body).await;

      // Then: should return Validation error
      assert!(result.is_err());
      match result.unwrap_err() {
        ModelError::Validation(msg) => {
          assert!(msg.contains("not implemented via generic handler"));
          assert!(msg.contains("POST /api/organizations"));
        }
        _ => panic!("Expected Validation error"),
      }
    }

    #[tokio::test]
    async fn should_reject_update_via_generic_handler() {
      // Given: Role RestModel and a test database
      let db = test_db().await;
      let id = Uuid::new_v4();
      let body = serde_json::json!({});

      // When: attempting to update a role via generic handler
      let result = Role::update(&db, id, body).await;

      // Then: should return Validation error
      assert!(result.is_err());
      match result.unwrap_err() {
        ModelError::Validation(msg) => {
          assert!(msg.contains("not implemented via generic handler"));
          assert!(msg.contains("PATCH /api/organizations"));
        }
        _ => panic!("Expected Validation error"),
      }
    }

    #[tokio::test]
    async fn should_reject_delete_via_generic_handler() {
      // Given: Role RestModel and a test database
      let db = test_db().await;
      let id = Uuid::new_v4();

      // When: attempting to delete a role via generic handler
      let result = Role::delete(&db, id).await;

      // Then: should return Validation error
      assert!(result.is_err());
      match result.unwrap_err() {
        ModelError::Validation(msg) => {
          assert!(msg.contains("not implemented via generic handler"));
          assert!(msg.contains("DELETE /api/organizations"));
        }
        _ => panic!("Expected Validation error"),
      }
    }
  }

  mod model_structure_behavior {
    use super::*;
    use crate::models::organization;

    async fn create_test_org(db: &forge_db::DbConnection, org_id: Uuid) {
      organization::Entity::insert(organization::ActiveModel {
        id: Set(org_id),
        name: Set(format!("Test Org {}", org_id)),
        slug: Set(format!("test-org-{}", org_id)),
        created_at: Set(Utc::now().naive_utc()),
        updated_at: Set(Utc::now().naive_utc()),
      })
      .exec(db)
      .await
      .expect("create test org");
    }

    #[tokio::test]
    async fn should_create_org_role_with_required_fields() {
      // Given: a test database with an organization
      let db = test_db().await;
      let org_id = Uuid::new_v4();
      create_test_org(&db, org_id).await;
      let role_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();

      // When: inserting an org_role with required fields
      let result = Entity::insert(ActiveModel {
        id: Set(role_id),
        org_id: Set(org_id),
        name: Set("viewer".to_string()),
        display_name: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await;

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_create_org_role_with_display_name() {
      // Given: a test database with an organization
      let db = test_db().await;
      let org_id = Uuid::new_v4();
      create_test_org(&db, org_id).await;
      let role_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();

      // When: inserting an org_role with display_name
      Entity::insert(ActiveModel {
        id: Set(role_id),
        org_id: Set(org_id),
        name: Set("editor".to_string()),
        display_name: Set(Some("Editor".to_string())),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert");

      // Then: should be able to retrieve it
      let role = Entity::find_by_id(role_id)
        .one(&db)
        .await
        .expect("find")
        .expect("role should exist");
      assert_eq!(role.id, role_id);
      assert_eq!(role.org_id, org_id);
      assert_eq!(role.name, "editor");
      assert_eq!(role.display_name, Some("Editor".to_string()));
    }

    #[tokio::test]
    async fn should_query_org_roles_by_org_id() {
      // Given: a test database with multiple organizations and org roles
      let db = test_db().await;
      let org1_id = Uuid::new_v4();
      let org2_id = Uuid::new_v4();
      create_test_org(&db, org1_id).await;
      create_test_org(&db, org2_id).await;
      let now = Utc::now().naive_utc();

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        org_id: Set(org1_id),
        name: Set("viewer".to_string()),
        display_name: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert role1");

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        org_id: Set(org1_id),
        name: Set("editor".to_string()),
        display_name: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert role2");

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        org_id: Set(org2_id),
        name: Set("viewer".to_string()),
        display_name: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert role3");

      // When: querying roles by org1_id
      let org1_roles = Entity::find()
        .filter(Column::OrgId.eq(org1_id))
        .all(&db)
        .await
        .expect("query");

      // Then: should find 2 roles for org1
      assert_eq!(org1_roles.len(), 2);
      assert!(org1_roles.iter().all(|r| r.org_id == org1_id));
    }

    #[tokio::test]
    async fn should_query_org_role_by_name() {
      // Given: a test database with an organization and org roles
      let db = test_db().await;
      let org_id = Uuid::new_v4();
      create_test_org(&db, org_id).await;
      let now = Utc::now().naive_utc();

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        org_id: Set(org_id),
        name: Set("admin".to_string()),
        display_name: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert");

      // When: querying role by name
      let role = Entity::find()
        .filter(Column::OrgId.eq(org_id))
        .filter(Column::Name.eq("admin"))
        .one(&db)
        .await
        .expect("query");

      // Then: should find the role
      assert!(role.is_some());
      let r = role.unwrap();
      assert_eq!(r.name, "admin");
      assert_eq!(r.org_id, org_id);
    }
  }

  mod row_to_json_behavior {
    use super::*;

    #[test]
    fn should_convert_model_to_json() {
      // Given: an org_role model
      let id = Uuid::new_v4();
      let org_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();
      let model = Model {
        id,
        org_id,
        name: "viewer".to_string(),
        display_name: Some("Viewer".to_string()),
        created_at: now,
        updated_at: now,
      };

      // When: converting to JSON
      let json = Role::row_to_json(&model);

      // Then: should have all expected fields
      assert_eq!(json["id"].as_str().unwrap(), id.to_string());
      assert_eq!(json["org_id"].as_str().unwrap(), org_id.to_string());
      assert_eq!(json["name"].as_str().unwrap(), "viewer");
      assert_eq!(json["display_name"].as_str().unwrap(), "Viewer");
      assert!(json["created_at"].is_string());
      assert!(json["updated_at"].is_string());
    }

    #[test]
    fn should_handle_none_display_name_in_json() {
      // Given: an org_role model with None display_name
      let id = Uuid::new_v4();
      let org_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();
      let model = Model {
        id,
        org_id,
        name: "editor".to_string(),
        display_name: None,
        created_at: now,
        updated_at: now,
      };

      // When: converting to JSON
      let json = Role::row_to_json(&model);

      // Then: display_name should be null
      assert!(json["display_name"].is_null());
    }
  }
}
