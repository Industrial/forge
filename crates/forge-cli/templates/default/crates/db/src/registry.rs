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

/// Returns true if model_id is served by the generic handler (has a RestModel impl and dispatch arm).
pub fn is_known_model(model_id: &str) -> bool {
  match model_id {
    "organization" | "user" | "role" | "permission" | "audit" => true,
    _ => false,
  }
}

/// Allowed filter fields for list query. Empty if unknown model.
pub fn effective_filter_fields(model_id: &str) -> &'static [&'static str] {
  match model_id {
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
  match model_id {
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
  match model_id {
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
  match model_id {
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
  match model_id {
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
  match model_id {
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
  match model_id {
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
  match model_id {
    "organization" => Organization::delete(db, id).await,
    "user" => User::delete(db, id).await,
    "role" => Role::delete(db, id).await,
    "permission" => Permission::delete(db, id).await,
    "audit" => Audit::delete(db, id).await,
    _ => Err(ModelError::UnknownModel),
  }
}
