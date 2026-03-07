//! Organization model (SeaORM) and RestModel impl for the generic REST handler.

use async_trait::async_trait;
use forge_db::DbConnection;
use sea_orm::entity::prelude::*;
use sea_orm::{QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model_error::ModelError;
use crate::organization::{
  CreateOrganizationBody, UpdateOrganizationBody, create_organization_impl,
  delete_organization_impl, update_organization_impl,
};
use crate::query_spec::{FilterCond, FilterOperator, SortDirection, SortSpec};
use crate::rest_model::{REST_ACTIONS, RestModel};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "organization")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  pub name: String,
  #[sea_orm(unique)]
  pub slug: String,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// ---- RestModel ----

const FILTER_SORT_RESPONSE: &[&str] = &["id", "name", "slug", "created_at", "updated_at"];

/// REST resource for the organization table; implements [RestModel].
pub struct Organization;

#[async_trait]
impl RestModel for Organization {
  type Entity = Entity;
  type Model = Model;

  fn model_id() -> &'static str
  where
    Self: Sized,
  {
    "organization"
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
    Some("Organization")
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
      "name": r.name,
      "slug": r.slug,
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
      "name" => (Column::Name, sort.direction),
      "slug" => (Column::Slug, sort.direction),
      "created_at" => (Column::CreatedAt, sort.direction),
      "updated_at" => (Column::UpdatedAt, sort.direction),
      _ => (Column::Name, sort.direction),
    };
    match dir {
      SortDirection::Asc => select.order_by_asc(col),
      SortDirection::Desc => select.order_by_desc(col),
    }
  }

  async fn create(db: &DbConnection, body: serde_json::Value) -> Result<Uuid, ModelError> {
    let payload: CreateOrganizationBody =
      serde_json::from_value(body).map_err(|e| ModelError::Validation(e.to_string()))?;
    create_organization_impl(db, &payload).await
  }

  async fn update(
    db: &DbConnection,
    id: Uuid,
    body: serde_json::Value,
  ) -> Result<serde_json::Value, ModelError> {
    let payload: UpdateOrganizationBody =
      serde_json::from_value(body).map_err(|e| ModelError::Validation(e.to_string()))?;
    let updated = update_organization_impl(db, id, &payload).await?;
    Ok(Self::row_to_json(&updated))
  }

  async fn delete(db: &DbConnection, id: Uuid) -> Result<bool, ModelError> {
    delete_organization_impl(db, id).await
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
    "name" | "slug" => {
      let col = if cond.field == "name" {
        Column::Name
      } else {
        Column::Slug
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
    fn should_have_model_id_organization() {
      // Given: Organization RestModel implementation
      // When: checking model_id
      // Then: should return "organization"
      assert_eq!(Organization::model_id(), "organization");
    }

    #[test]
    fn should_have_display_name() {
      // Given: Organization RestModel implementation
      // When: checking display_name
      // Then: should return Some("Organization")
      assert_eq!(Organization::display_name(), Some("Organization"));
    }

    #[tokio::test]
    async fn should_create_organization_via_rest_model() {
      // Given: Organization RestModel and a test database
      let db = test_db().await;
      let body = serde_json::json!({
        "name": "Test Org",
        "slug": "test-org"
      });

      // When: creating an organization
      let result = Organization::create(&db, body).await;

      // Then: should return a UUID
      assert!(result.is_ok());
      let org_id = result.unwrap();
      assert_ne!(org_id, Uuid::nil());
    }

    #[tokio::test]
    async fn should_update_organization_via_rest_model() {
      // Given: an existing organization
      let db = test_db().await;
      let org_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();

      Entity::insert(ActiveModel {
        id: Set(org_id),
        name: Set("Original Name".to_string()),
        slug: Set("original-slug".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert");

      let body = serde_json::json!({
        "name": "Updated Name",
        "slug": "updated-slug"
      });

      // When: updating the organization
      let result = Organization::update(&db, org_id, body).await;

      // Then: should return updated JSON
      assert!(result.is_ok());
      let json = result.unwrap();
      assert_eq!(json["name"].as_str().unwrap(), "Updated Name");
      assert_eq!(json["slug"].as_str().unwrap(), "updated-slug");
    }

    #[tokio::test]
    async fn should_delete_organization_via_rest_model() {
      // Given: an existing organization
      let db = test_db().await;
      let org_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();

      Entity::insert(ActiveModel {
        id: Set(org_id),
        name: Set("Test Org".to_string()),
        slug: Set("test-org".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert");

      // When: deleting the organization
      let result = Organization::delete(&db, org_id).await;

      // Then: should return true
      assert!(result.is_ok());
      assert!(result.unwrap());
    }
  }

  mod model_structure_behavior {
    use super::*;

    #[tokio::test]
    async fn should_create_organization_with_required_fields() {
      // Given: a test database
      let db = test_db().await;
      let org_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();

      // When: inserting an organization with required fields
      let result = Entity::insert(ActiveModel {
        id: Set(org_id),
        name: Set("Test Organization".to_string()),
        slug: Set("test-org".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await;

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_store_organization_with_name_and_slug() {
      // Given: a test database
      let db = test_db().await;
      let org_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();

      // When: inserting an organization
      Entity::insert(ActiveModel {
        id: Set(org_id),
        name: Set("My Organization".to_string()),
        slug: Set("my-org".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert");

      // Then: should be able to retrieve it
      let org = Entity::find_by_id(org_id)
        .one(&db)
        .await
        .expect("find")
        .expect("org should exist");
      assert_eq!(org.id, org_id);
      assert_eq!(org.name, "My Organization");
      assert_eq!(org.slug, "my-org");
    }

    #[tokio::test]
    async fn should_query_organization_by_slug() {
      // Given: a test database with organizations
      let db = test_db().await;
      let now = Utc::now().naive_utc();

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set("Org 1".to_string()),
        slug: Set("org-1".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert org1");

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set("Org 2".to_string()),
        slug: Set("org-2".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert org2");

      // When: querying by slug
      let org = Entity::find()
        .filter(Column::Slug.eq("org-1"))
        .one(&db)
        .await
        .expect("query");

      // Then: should find the correct organization
      assert!(org.is_some());
      let o = org.unwrap();
      assert_eq!(o.slug, "org-1");
      assert_eq!(o.name, "Org 1");
    }
  }

  mod row_to_json_behavior {
    use super::*;

    #[test]
    fn should_convert_model_to_json() {
      // Given: an organization model
      let id = Uuid::new_v4();
      let now = Utc::now().naive_utc();
      let model = Model {
        id,
        name: "Test Org".to_string(),
        slug: "test-org".to_string(),
        created_at: now,
        updated_at: now,
      };

      // When: converting to JSON
      let json = Organization::row_to_json(&model);

      // Then: should have all expected fields
      assert_eq!(json["id"].as_str().unwrap(), id.to_string());
      assert_eq!(json["name"].as_str().unwrap(), "Test Org");
      assert_eq!(json["slug"].as_str().unwrap(), "test-org");
      assert!(json["created_at"].is_string());
      assert!(json["updated_at"].is_string());
    }
  }
}
