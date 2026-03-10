//! Registry of models served by the generic REST handler.
//! Dispatch is by model_id. Add a model: implement [RestModel] in [models], re-export here, add match arms below.

use forge_db::DbConnection;
use std::sync::OnceLock;
use uuid::Uuid;

use crate::model_error::ModelError;
use crate::query_spec::ListQuerySpec;
use crate::rest_model::RestModel;

// Re-exports: one per model that implements RestModel.
pub use crate::models::audit_log::Audit;
pub use crate::models::org_role::Role;
pub use crate::models::organization::Organization;
pub use crate::models::role_permission::Permission;
pub use crate::models::user::User;

/// Normalize entity_id: "audit_log" (frontend/RPC) maps to "audit" (RestModel model_id).
fn normalize_entity_id(model_id: &str) -> &str {
  if model_id == "audit_log" {
    "audit"
  } else {
    model_id
  }
}

/// Returns true if model_id is served by the generic handler (has a RestModel impl and dispatch arm).
pub fn is_known_model(model_id: &str) -> bool {
  matches!(
    model_id,
    "organization" | "user" | "role" | "permission" | "audit" | "audit_log"
  )
}

/// Allowed filter fields for list query. Empty if unknown model.
pub fn effective_filter_fields(model_id: &str) -> &'static [&'static str] {
  match normalize_entity_id(model_id) {
    "organization" => Organization::filter_fields(),
    "user" => User::filter_fields(),
    "role" => Role::filter_fields(),
    "permission" => Permission::filter_fields(),
    "audit" => Audit::filter_fields(),
    _ => &[],
  }
}

/// Allowed sort fields for list query. Empty if unknown model.
pub fn effective_sort_fields(model_id: &str) -> &'static [&'static str] {
  match normalize_entity_id(model_id) {
    "organization" => Organization::sort_fields(),
    "user" => User::sort_fields(),
    "role" => Role::sort_fields(),
    "permission" => Permission::sort_fields(),
    "audit" => Audit::sort_fields(),
    _ => &[],
  }
}

/// Response column names for list/get. Empty if unknown model.
pub fn effective_response_columns(model_id: &str) -> Vec<&'static str> {
  match normalize_entity_id(model_id) {
    "organization" => Organization::response_columns().to_vec(),
    "user" => User::response_columns().to_vec(),
    "role" => Role::response_columns().to_vec(),
    "permission" => Permission::response_columns().to_vec(),
    "audit" => Audit::response_columns().to_vec(),
    _ => vec![],
  }
}

static ALLOWED_KEYS: OnceLock<Vec<&'static str>> = OnceLock::new();

/// Permission keys for all registered models: `<model_id>.<action>` plus `all.read`, `all.write`.
/// Derived from RestModel implementors.
pub fn allowed_permission_keys() -> &'static [&'static str] {
  ALLOWED_KEYS.get_or_init(|| {
    let model_actions: &[(&str, &[&str])] = &[
      (Organization::model_id(), Organization::supported_actions()),
      (User::model_id(), User::supported_actions()),
      (Role::model_id(), Role::supported_actions()),
      (Permission::model_id(), Permission::supported_actions()),
      (Audit::model_id(), Audit::supported_actions()),
    ];
    let mut keys: Vec<&'static str> = model_actions
      .iter()
      .flat_map(|(id, actions)| {
        actions.iter().map(move |action| {
          let s = format!("{}.{}", id, action);
          Box::leak(s.into_boxed_str()) as &'static str
        })
      })
      .collect();
    keys.push("all.read");
    keys.push("all.write");
    keys
  })
}

/// List models; returns `{ "data": [ ... ] }`. Err(UnknownModel) if model_id not registered.
pub async fn list_models(
  model_id: &str,
  db: &DbConnection,
  spec: &ListQuerySpec,
) -> Result<serde_json::Value, ModelError> {
  match normalize_entity_id(model_id) {
    "organization" => Organization::list(db, spec).await,
    "user" => User::list(db, spec).await,
    "role" => Role::list(db, spec).await,
    "permission" => Permission::list(db, spec).await,
    "audit" => Audit::list(db, spec).await,
    _ => Err(ModelError::UnknownModel),
  }
}

/// Get one model by id. Ok(None) if not found; Err(UnknownModel) if model_id not registered.
pub async fn get_model(
  model_id: &str,
  db: &DbConnection,
  id: Uuid,
) -> Result<Option<serde_json::Value>, ModelError> {
  match normalize_entity_id(model_id) {
    "organization" => Organization::get(db, id).await,
    "user" => User::get(db, id).await,
    "role" => Role::get(db, id).await,
    "permission" => Permission::get(db, id).await,
    "audit" => Audit::get(db, id).await,
    _ => Err(ModelError::UnknownModel),
  }
}

/// Create model; returns new id. Err(UnknownModel) if model_id not registered.
pub async fn create_model(
  model_id: &str,
  db: &DbConnection,
  body: serde_json::Value,
) -> Result<Uuid, ModelError> {
  match normalize_entity_id(model_id) {
    "organization" => Organization::create(db, body).await,
    "user" => User::create(db, body).await,
    "role" => Role::create(db, body).await,
    "permission" => Permission::create(db, body).await,
    "audit" => Audit::create(db, body).await,
    _ => Err(ModelError::UnknownModel),
  }
}

/// Update model by id. Err(UnknownModel) if model_id not registered.
pub async fn update_model(
  model_id: &str,
  db: &DbConnection,
  id: Uuid,
  body: serde_json::Value,
) -> Result<serde_json::Value, ModelError> {
  match normalize_entity_id(model_id) {
    "organization" => Organization::update(db, id, body).await,
    "user" => User::update(db, id, body).await,
    "role" => Role::update(db, id, body).await,
    "permission" => Permission::update(db, id, body).await,
    "audit" => Audit::update(db, id, body).await,
    _ => Err(ModelError::UnknownModel),
  }
}

/// Delete model by id. Ok(false) if not found; Err(UnknownModel) if model_id not registered.
pub async fn delete_model(model_id: &str, db: &DbConnection, id: Uuid) -> Result<bool, ModelError> {
  match normalize_entity_id(model_id) {
    "organization" => Organization::delete(db, id).await,
    "user" => User::delete(db, id).await,
    "role" => Role::delete(db, id).await,
    "permission" => Permission::delete(db, id).await,
    "audit" => Audit::delete(db, id).await,
    _ => Err(ModelError::UnknownModel),
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

  mod is_known_model_behavior {
    use super::*;

    #[test]
    fn should_return_true_for_registered_models() {
      // Given: registry with registered models
      // When: checking known model IDs
      // Then: should return true
      assert!(is_known_model("organization"));
      assert!(is_known_model("user"));
      assert!(is_known_model("role"));
      assert!(is_known_model("permission"));
      assert!(is_known_model("audit"));
      assert!(is_known_model("audit_log"));
    }

    #[test]
    fn should_return_false_for_unknown_models() {
      // Given: registry
      // When: checking unknown model ID
      // Then: should return false
      assert!(!is_known_model("unknown_model"));
      assert!(!is_known_model(""));
      assert!(!is_known_model("invalid"));
    }
  }

  mod effective_filter_fields_behavior {
    use super::*;

    #[test]
    fn should_return_filter_fields_for_known_models() {
      // Given: registry
      // When: getting filter fields for known model
      // Then: should return non-empty array
      let org_fields = effective_filter_fields("organization");
      assert!(!org_fields.is_empty());
      assert!(org_fields.contains(&"id"));
      assert!(org_fields.contains(&"name"));
    }

    #[test]
    fn should_return_empty_for_unknown_models() {
      // Given: registry
      // When: getting filter fields for unknown model
      // Then: should return empty array
      let fields = effective_filter_fields("unknown_model");
      assert!(fields.is_empty());
    }
  }

  mod effective_sort_fields_behavior {
    use super::*;

    #[test]
    fn should_return_sort_fields_for_known_models() {
      // Given: registry
      // When: getting sort fields for known model
      // Then: should return non-empty array
      let org_fields = effective_sort_fields("organization");
      assert!(!org_fields.is_empty());
    }

    #[test]
    fn should_return_empty_for_unknown_models() {
      // Given: registry
      // When: getting sort fields for unknown model
      // Then: should return empty array
      let fields = effective_sort_fields("unknown_model");
      assert!(fields.is_empty());
    }
  }

  mod effective_response_columns_behavior {
    use super::*;

    #[test]
    fn should_return_response_columns_for_known_models() {
      // Given: registry
      // When: getting response columns for known model
      // Then: should return non-empty vector
      let org_cols = effective_response_columns("organization");
      assert!(!org_cols.is_empty());
      assert!(org_cols.contains(&"id"));
    }

    #[test]
    fn should_return_empty_for_unknown_models() {
      // Given: registry
      // When: getting response columns for unknown model
      // Then: should return empty vector
      let cols = effective_response_columns("unknown_model");
      assert!(cols.is_empty());
    }
  }

  mod allowed_permission_keys_behavior {
    use super::*;

    #[test]
    fn should_include_model_action_permissions() {
      // Given: registry
      // When: getting allowed permission keys
      // Then: should include model.action format
      let keys = allowed_permission_keys();
      assert!(keys.contains(&"organization.read"));
      assert!(keys.contains(&"user.read"));
      assert!(keys.contains(&"audit.read"));
    }

    #[test]
    fn should_include_all_read_and_all_write() {
      // Given: registry
      // When: getting allowed permission keys
      // Then: should include all.read and all.write
      let keys = allowed_permission_keys();
      assert!(keys.contains(&"all.read"));
      assert!(keys.contains(&"all.write"));
    }

    #[test]
    fn should_return_consistent_results() {
      // Given: registry
      // When: calling allowed_permission_keys multiple times
      // Then: should return same result (cached)
      let keys1 = allowed_permission_keys();
      let keys2 = allowed_permission_keys();
      assert_eq!(keys1.len(), keys2.len());
    }
  }

  mod list_models_behavior {
    use super::*;

    #[tokio::test]
    async fn should_list_organizations() {
      // Given: a test database with organizations
      let db = test_db().await;
      let now = chrono::Utc::now().naive_utc();
      crate::models::organization::Entity::insert(crate::models::organization::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set("Org 1".to_string()),
        slug: Set("org-1".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert");

      let spec = ListQuerySpec::default();

      // When: listing organizations
      let result = list_models("organization", &db, &spec).await;

      // Then: should return JSON with data array
      assert!(result.is_ok());
      let json = result.unwrap();
      assert!(json.get("data").and_then(|d| d.as_array()).is_some());
    }

    #[tokio::test]
    async fn should_return_unknown_model_error_for_invalid_id() {
      // Given: a test database
      let db = test_db().await;
      let spec = ListQuerySpec::default();

      // When: listing unknown model
      let result = list_models("unknown_model", &db, &spec).await;

      // Then: should return UnknownModel error
      assert!(result.is_err());
      match result.unwrap_err() {
        ModelError::UnknownModel => {}
        _ => panic!("Expected UnknownModel error"),
      }
    }
  }

  mod get_model_behavior {
    use super::*;

    #[tokio::test]
    async fn should_get_existing_organization() {
      // Given: a test database with organization
      let db = test_db().await;
      let org_id = Uuid::new_v4();
      let now = chrono::Utc::now().naive_utc();
      crate::models::organization::Entity::insert(crate::models::organization::ActiveModel {
        id: Set(org_id),
        name: Set("Test Org".to_string()),
        slug: Set("test-org".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert");

      // When: getting organization
      let result = get_model("organization", &db, org_id).await;

      // Then: should return Some(JSON)
      assert!(result.is_ok());
      let json_opt = result.unwrap();
      assert!(json_opt.is_some());
      let json = json_opt.unwrap();
      assert_eq!(json["id"].as_str().unwrap(), org_id.to_string());
    }

    #[tokio::test]
    async fn should_return_none_for_non_existent_model() {
      // Given: a test database
      let db = test_db().await;
      let non_existent_id = Uuid::new_v4();

      // When: getting non-existent organization
      let result = get_model("organization", &db, non_existent_id).await;

      // Then: should return Ok(None)
      assert!(result.is_ok());
      assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn should_return_unknown_model_error_for_invalid_id() {
      // Given: a test database
      let db = test_db().await;
      let id = Uuid::new_v4();

      // When: getting unknown model
      let result = get_model("unknown_model", &db, id).await;

      // Then: should return UnknownModel error
      assert!(result.is_err());
      match result.unwrap_err() {
        ModelError::UnknownModel => {}
        _ => panic!("Expected UnknownModel error"),
      }
    }
  }

  mod create_model_behavior {
    use super::*;

    #[tokio::test]
    async fn should_create_organization_via_registry() {
      // Given: a test database
      let db = test_db().await;
      let body = serde_json::json!({
        "name": "Registry Org",
        "slug": "registry-org"
      });

      // When: creating organization via registry
      let result = create_model("organization", &db, body).await;

      // Then: should return UUID
      assert!(result.is_ok());
      let org_id = result.unwrap();
      assert_ne!(org_id, Uuid::nil());
    }

    #[tokio::test]
    async fn should_return_unknown_model_error_for_invalid_id() {
      // Given: a test database
      let db = test_db().await;
      let body = serde_json::json!({});

      // When: creating unknown model
      let result = create_model("unknown_model", &db, body).await;

      // Then: should return UnknownModel error
      assert!(result.is_err());
      match result.unwrap_err() {
        ModelError::UnknownModel => {}
        _ => panic!("Expected UnknownModel error"),
      }
    }
  }

  mod update_model_behavior {
    use super::*;

    #[tokio::test]
    async fn should_update_organization_via_registry() {
      // Given: an existing organization
      let db = test_db().await;
      let org_id = Uuid::new_v4();
      let now = chrono::Utc::now().naive_utc();
      crate::models::organization::Entity::insert(crate::models::organization::ActiveModel {
        id: Set(org_id),
        name: Set("Original".to_string()),
        slug: Set("original".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert");

      let body = serde_json::json!({
        "name": "Updated"
      });

      // When: updating organization via registry
      let result = update_model("organization", &db, org_id, body).await;

      // Then: should return updated JSON
      assert!(result.is_ok());
      let json = result.unwrap();
      assert_eq!(json["name"].as_str().unwrap(), "Updated");
    }

    #[tokio::test]
    async fn should_return_unknown_model_error_for_invalid_id() {
      // Given: a test database
      let db = test_db().await;
      let id = Uuid::new_v4();
      let body = serde_json::json!({});

      // When: updating unknown model
      let result = update_model("unknown_model", &db, id, body).await;

      // Then: should return UnknownModel error
      assert!(result.is_err());
      match result.unwrap_err() {
        ModelError::UnknownModel => {}
        _ => panic!("Expected UnknownModel error"),
      }
    }
  }

  mod delete_model_behavior {
    use super::*;

    #[tokio::test]
    async fn should_delete_organization_via_registry() {
      // Given: an existing organization
      let db = test_db().await;
      let org_id = Uuid::new_v4();
      let now = chrono::Utc::now().naive_utc();
      crate::models::organization::Entity::insert(crate::models::organization::ActiveModel {
        id: Set(org_id),
        name: Set("To Delete".to_string()),
        slug: Set("to-delete".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert");

      // When: deleting organization via registry
      let result = delete_model("organization", &db, org_id).await;

      // Then: should return true
      assert!(result.is_ok());
      assert!(result.unwrap());
    }

    #[tokio::test]
    async fn should_return_false_for_non_existent_model() {
      // Given: a test database
      let db = test_db().await;
      let non_existent_id = Uuid::new_v4();

      // When: deleting non-existent organization
      let result = delete_model("organization", &db, non_existent_id).await;

      // Then: should return false
      assert!(result.is_ok());
      assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn should_return_unknown_model_error_for_invalid_id() {
      // Given: a test database
      let db = test_db().await;
      let id = Uuid::new_v4();

      // When: deleting unknown model
      let result = delete_model("unknown_model", &db, id).await;

      // Then: should return UnknownModel error
      assert!(result.is_err());
      match result.unwrap_err() {
        ModelError::UnknownModel => {}
        _ => panic!("Expected UnknownModel error"),
      }
    }
  }
}
