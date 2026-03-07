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
    fn should_have_model_id_audit() {
      // Given: Audit RestModel implementation
      // When: checking model_id
      // Then: should return "audit"
      assert_eq!(Audit::model_id(), "audit");
    }

    #[test]
    fn should_have_display_name() {
      // Given: Audit RestModel implementation
      // When: checking display_name
      // Then: should return Some("Audit log")
      assert_eq!(Audit::display_name(), Some("Audit log"));
    }

    #[test]
    fn should_support_only_read_action() {
      // Given: Audit RestModel implementation
      // When: checking supported_actions
      // Then: should return only ["read"]
      assert_eq!(Audit::supported_actions(), &["read"]);
    }

    #[tokio::test]
    async fn should_reject_create_operation() {
      // Given: Audit RestModel and a test database
      let db = test_db().await;
      let body = serde_json::json!({});

      // When: attempting to create an audit log entry
      let result = Audit::create(&db, body).await;

      // Then: should return Validation error
      assert!(result.is_err());
      match result.unwrap_err() {
        ModelError::Validation(msg) => {
          assert!(msg.contains("read-only"));
          assert!(msg.contains("create not supported"));
        }
        _ => panic!("Expected Validation error"),
      }
    }

    #[tokio::test]
    async fn should_reject_update_operation() {
      // Given: Audit RestModel and a test database
      let db = test_db().await;
      let id = Uuid::new_v4();
      let body = serde_json::json!({});

      // When: attempting to update an audit log entry
      let result = Audit::update(&db, id, body).await;

      // Then: should return Validation error
      assert!(result.is_err());
      match result.unwrap_err() {
        ModelError::Validation(msg) => {
          assert!(msg.contains("read-only"));
          assert!(msg.contains("update not supported"));
        }
        _ => panic!("Expected Validation error"),
      }
    }

    #[tokio::test]
    async fn should_reject_delete_operation() {
      // Given: Audit RestModel and a test database
      let db = test_db().await;
      let id = Uuid::new_v4();

      // When: attempting to delete an audit log entry
      let result = Audit::delete(&db, id).await;

      // Then: should return Validation error
      assert!(result.is_err());
      match result.unwrap_err() {
        ModelError::Validation(msg) => {
          assert!(msg.contains("read-only"));
          assert!(msg.contains("delete not supported"));
        }
        _ => panic!("Expected Validation error"),
      }
    }
  }

  mod model_structure_behavior {
    use super::*;

    #[tokio::test]
    async fn should_create_audit_log_entry() {
      // Given: a test database
      let db = test_db().await;
      let id = Uuid::new_v4();
      let actor_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();

      // When: inserting an audit log entry
      let result = Entity::insert(ActiveModel {
        id: Set(id),
        event_kind: Set("test".to_string()),
        actor_id: Set(actor_id),
        subject_id: Set(None),
        organization_id: Set(None),
        action: Set("create".to_string()),
        resource_type: Set("user".to_string()),
        resource_id: Set(None),
        outcome: Set("success".to_string()),
        reason: Set(None),
        occurred_at: Set(now),
      })
      .exec(&db)
      .await;

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_store_all_audit_log_fields() {
      // Given: a test database
      let db = test_db().await;
      let id = Uuid::new_v4();
      let actor_id = Uuid::new_v4();
      let subject_id = Uuid::new_v4();
      let org_id = Uuid::new_v4();
      let resource_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();

      // When: inserting an audit log entry with all fields
      Entity::insert(ActiveModel {
        id: Set(id),
        event_kind: Set("user.created".to_string()),
        actor_id: Set(actor_id),
        subject_id: Set(Some(subject_id)),
        organization_id: Set(Some(org_id)),
        action: Set("create".to_string()),
        resource_type: Set("user".to_string()),
        resource_id: Set(Some(resource_id)),
        outcome: Set("success".to_string()),
        reason: Set(Some("User registration".to_string())),
        occurred_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert");

      // Then: should be able to retrieve it with all fields
      let entry = Entity::find_by_id(id)
        .one(&db)
        .await
        .expect("find")
        .expect("entry should exist");
      assert_eq!(entry.id, id);
      assert_eq!(entry.actor_id, actor_id);
      assert_eq!(entry.subject_id, Some(subject_id));
      assert_eq!(entry.organization_id, Some(org_id));
      assert_eq!(entry.resource_id, Some(resource_id));
      assert_eq!(entry.event_kind, "user.created");
      assert_eq!(entry.action, "create");
      assert_eq!(entry.resource_type, "user");
      assert_eq!(entry.outcome, "success");
      assert_eq!(entry.reason, Some("User registration".to_string()));
    }

    #[tokio::test]
    async fn should_query_audit_logs_by_actor_id() {
      // Given: a test database with multiple audit log entries
      let db = test_db().await;
      let actor1_id = Uuid::new_v4();
      let actor2_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        event_kind: Set("test".to_string()),
        actor_id: Set(actor1_id),
        subject_id: Set(None),
        organization_id: Set(None),
        action: Set("create".to_string()),
        resource_type: Set("user".to_string()),
        resource_id: Set(None),
        outcome: Set("success".to_string()),
        reason: Set(None),
        occurred_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert entry1");

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        event_kind: Set("test".to_string()),
        actor_id: Set(actor1_id),
        subject_id: Set(None),
        organization_id: Set(None),
        action: Set("update".to_string()),
        resource_type: Set("user".to_string()),
        resource_id: Set(None),
        outcome: Set("success".to_string()),
        reason: Set(None),
        occurred_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert entry2");

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        event_kind: Set("test".to_string()),
        actor_id: Set(actor2_id),
        subject_id: Set(None),
        organization_id: Set(None),
        action: Set("delete".to_string()),
        resource_type: Set("user".to_string()),
        resource_id: Set(None),
        outcome: Set("success".to_string()),
        reason: Set(None),
        occurred_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert entry3");

      // When: querying audit logs by actor1_id
      let actor1_logs = Entity::find()
        .filter(Column::ActorId.eq(actor1_id))
        .all(&db)
        .await
        .expect("query");

      // Then: should find 2 entries for actor1
      assert_eq!(actor1_logs.len(), 2);
      assert!(actor1_logs.iter().all(|e| e.actor_id == actor1_id));
    }
  }

  mod row_to_json_behavior {
    use super::*;

    #[test]
    fn should_convert_model_to_json() {
      // Given: an audit log model
      let id = Uuid::new_v4();
      let actor_id = Uuid::new_v4();
      let subject_id = Uuid::new_v4();
      let org_id = Uuid::new_v4();
      let resource_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();
      let model = Model {
        id,
        event_kind: "user.created".to_string(),
        actor_id,
        subject_id: Some(subject_id),
        organization_id: Some(org_id),
        action: "create".to_string(),
        resource_type: "user".to_string(),
        resource_id: Some(resource_id),
        outcome: "success".to_string(),
        reason: Some("User registration".to_string()),
        occurred_at: now,
      };

      // When: converting to JSON
      let json = Audit::row_to_json(&model);

      // Then: should have all expected fields
      assert_eq!(json["id"].as_str().unwrap(), id.to_string());
      assert_eq!(json["event_kind"].as_str().unwrap(), "user.created");
      assert_eq!(json["actor_id"].as_str().unwrap(), actor_id.to_string());
      assert_eq!(json["subject_id"].as_str().unwrap(), subject_id.to_string());
      assert_eq!(
        json["organization_id"].as_str().unwrap(),
        org_id.to_string()
      );
      assert_eq!(json["action"].as_str().unwrap(), "create");
      assert_eq!(json["resource_type"].as_str().unwrap(), "user");
      assert_eq!(
        json["resource_id"].as_str().unwrap(),
        resource_id.to_string()
      );
      assert_eq!(json["outcome"].as_str().unwrap(), "success");
      assert_eq!(json["reason"].as_str().unwrap(), "User registration");
      assert!(json["occurred_at"].is_string());
    }

    #[test]
    fn should_handle_optional_fields_in_json() {
      // Given: an audit log model with None optional fields
      let id = Uuid::new_v4();
      let actor_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();
      let model = Model {
        id,
        event_kind: "test".to_string(),
        actor_id,
        subject_id: None,
        organization_id: None,
        action: "create".to_string(),
        resource_type: "user".to_string(),
        resource_id: None,
        outcome: "success".to_string(),
        reason: None,
        occurred_at: now,
      };

      // When: converting to JSON
      let json = Audit::row_to_json(&model);

      // Then: optional fields should be null
      assert!(json["subject_id"].is_null());
      assert!(json["organization_id"].is_null());
      assert!(json["resource_id"].is_null());
      assert!(json["reason"].is_null());
    }
  }
}
