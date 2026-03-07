//! Role permission model (SeaORM) and RestModel impl for the generic REST handler.

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

const FILTER_SORT_RESPONSE: &[&str] = &["id", "scope", "role_name", "permission_key", "org_id"];

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

  fn apply_filter(select: sea_orm::Select<Entity>, cond: &FilterCond) -> sea_orm::Select<Entity> {
    apply_filter(select, cond)
  }

  fn default_sort(select: sea_orm::Select<Entity>) -> sea_orm::Select<Entity> {
    select.order_by_asc(Column::RoleName)
  }

  fn apply_sort(select: sea_orm::Select<Entity>, sort: &SortSpec) -> sea_orm::Select<Entity> {
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

  async fn create(_db: &DbConnection, _body: serde_json::Value) -> Result<Uuid, ModelError> {
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

#[cfg(test)]
mod bdd_tests {
  use super::*;
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
    fn should_have_model_id_permission() {
      // Given: Permission RestModel implementation
      // When: checking model_id
      // Then: should return "permission"
      assert_eq!(Permission::model_id(), "permission");
    }

    #[test]
    fn should_have_display_name() {
      // Given: Permission RestModel implementation
      // When: checking display_name
      // Then: should return Some("Permission")
      assert_eq!(Permission::display_name(), Some("Permission"));
    }

    #[tokio::test]
    async fn should_reject_create_via_generic_handler() {
      // Given: Permission RestModel and a test database
      let db = test_db().await;
      let body = serde_json::json!({});

      // When: attempting to create a permission via generic handler
      let result = Permission::create(&db, body).await;

      // Then: should return Validation error
      assert!(result.is_err());
      match result.unwrap_err() {
        ModelError::Validation(msg) => {
          assert!(msg.contains("not implemented via generic handler"));
        }
        _ => panic!("Expected Validation error"),
      }
    }

    #[tokio::test]
    async fn should_reject_update_via_generic_handler() {
      // Given: Permission RestModel and a test database
      let db = test_db().await;
      let id = Uuid::new_v4();
      let body = serde_json::json!({});

      // When: attempting to update a permission via generic handler
      let result = Permission::update(&db, id, body).await;

      // Then: should return Validation error
      assert!(result.is_err());
      match result.unwrap_err() {
        ModelError::Validation(msg) => {
          assert!(msg.contains("not implemented via generic handler"));
        }
        _ => panic!("Expected Validation error"),
      }
    }

    #[tokio::test]
    async fn should_reject_delete_via_generic_handler() {
      // Given: Permission RestModel and a test database
      let db = test_db().await;
      let id = Uuid::new_v4();

      // When: attempting to delete a permission via generic handler
      let result = Permission::delete(&db, id).await;

      // Then: should return Validation error
      assert!(result.is_err());
      match result.unwrap_err() {
        ModelError::Validation(msg) => {
          assert!(msg.contains("not implemented via generic handler"));
        }
        _ => panic!("Expected Validation error"),
      }
    }
  }

  mod model_structure_behavior {
    use super::*;

    #[tokio::test]
    async fn should_create_role_permission_with_required_fields() {
      // Given: a test database
      let db = test_db().await;
      let perm_id = Uuid::new_v4();
      let org_id = Uuid::new_v4();

      // When: inserting a role_permission with required fields
      let result = Entity::insert(ActiveModel {
        id: Set(perm_id),
        scope: Set("org".to_string()),
        role_name: Set("viewer".to_string()),
        permission_key: Set("dashboard.read".to_string()),
        org_id: Set(Some(org_id)),
      })
      .exec(&db)
      .await;

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_create_role_permission_with_global_scope() {
      // Given: a test database
      let db = test_db().await;
      let perm_id = Uuid::new_v4();

      // When: inserting a role_permission with global scope (org_id = None)
      Entity::insert(ActiveModel {
        id: Set(perm_id),
        scope: Set("global".to_string()),
        role_name: Set("platform_admin".to_string()),
        permission_key: Set("all.read".to_string()),
        org_id: Set(None),
      })
      .exec(&db)
      .await
      .expect("insert");

      // Then: should be able to retrieve it
      let perm = Entity::find_by_id(perm_id)
        .one(&db)
        .await
        .expect("find")
        .expect("permission should exist");
      assert_eq!(perm.id, perm_id);
      assert_eq!(perm.scope, "global");
      assert_eq!(perm.org_id, None);
    }

    #[tokio::test]
    async fn should_query_role_permissions_by_role_name() {
      // Given: a test database with role permissions
      let db = test_db().await;
      let org_id = Uuid::new_v4();

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        scope: Set("org".to_string()),
        role_name: Set("viewer".to_string()),
        permission_key: Set("dashboard.read".to_string()),
        org_id: Set(Some(org_id)),
      })
      .exec(&db)
      .await
      .expect("insert perm1");

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        scope: Set("org".to_string()),
        role_name: Set("viewer".to_string()),
        permission_key: Set("dashboard.audit.read".to_string()),
        org_id: Set(Some(org_id)),
      })
      .exec(&db)
      .await
      .expect("insert perm2");

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        scope: Set("org".to_string()),
        role_name: Set("editor".to_string()),
        permission_key: Set("dashboard.write".to_string()),
        org_id: Set(Some(org_id)),
      })
      .exec(&db)
      .await
      .expect("insert perm3");

      // When: querying permissions by role_name
      let viewer_perms = Entity::find()
        .filter(Column::RoleName.eq("viewer"))
        .all(&db)
        .await
        .expect("query");

      // Then: should find 2 permissions for viewer role
      assert_eq!(viewer_perms.len(), 2);
      assert!(viewer_perms.iter().all(|p| p.role_name == "viewer"));
    }

    #[tokio::test]
    async fn should_query_role_permissions_by_org_id() {
      // Given: a test database with role permissions
      let db = test_db().await;
      let org1_id = Uuid::new_v4();
      let org2_id = Uuid::new_v4();

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        scope: Set("org".to_string()),
        role_name: Set("viewer".to_string()),
        permission_key: Set("dashboard.read".to_string()),
        org_id: Set(Some(org1_id)),
      })
      .exec(&db)
      .await
      .expect("insert perm1");

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        scope: Set("org".to_string()),
        role_name: Set("editor".to_string()),
        permission_key: Set("dashboard.write".to_string()),
        org_id: Set(Some(org1_id)),
      })
      .exec(&db)
      .await
      .expect("insert perm2");

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        scope: Set("org".to_string()),
        role_name: Set("viewer".to_string()),
        permission_key: Set("dashboard.read".to_string()),
        org_id: Set(Some(org2_id)),
      })
      .exec(&db)
      .await
      .expect("insert perm3");

      // When: querying permissions by org1_id
      let org1_perms = Entity::find()
        .filter(Column::OrgId.eq(Some(org1_id)))
        .all(&db)
        .await
        .expect("query");

      // Then: should find 2 permissions for org1
      assert_eq!(org1_perms.len(), 2);
      assert!(org1_perms.iter().all(|p| p.org_id == Some(org1_id)));
    }
  }

  mod row_to_json_behavior {
    use super::*;

    #[test]
    fn should_convert_model_to_json() {
      // Given: a role_permission model
      let id = Uuid::new_v4();
      let org_id = Uuid::new_v4();
      let model = Model {
        id,
        scope: "org".to_string(),
        role_name: "viewer".to_string(),
        permission_key: "dashboard.read".to_string(),
        org_id: Some(org_id),
      };

      // When: converting to JSON
      let json = Permission::row_to_json(&model);

      // Then: should have all expected fields
      assert_eq!(json["id"].as_str().unwrap(), id.to_string());
      assert_eq!(json["scope"].as_str().unwrap(), "org");
      assert_eq!(json["role_name"].as_str().unwrap(), "viewer");
      assert_eq!(json["permission_key"].as_str().unwrap(), "dashboard.read");
      assert_eq!(json["org_id"].as_str().unwrap(), org_id.to_string());
    }

    #[test]
    fn should_handle_none_org_id_in_json() {
      // Given: a role_permission model with None org_id
      let id = Uuid::new_v4();
      let model = Model {
        id,
        scope: "global".to_string(),
        role_name: "platform_admin".to_string(),
        permission_key: "all.read".to_string(),
        org_id: None,
      };

      // When: converting to JSON
      let json = Permission::row_to_json(&model);

      // Then: org_id should be null
      assert!(json["org_id"].is_null());
    }
  }
}
