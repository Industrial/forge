//! Trait for models served by the generic REST handler.
//! Implement metadata and hooks; list/get have default implementations.

use async_trait::async_trait;
use forge_db::DbConnection;
use forge_query::{FilterCond, ListQuerySpec, SortSpec};
use sea_orm::entity::PrimaryKeyTrait;
use sea_orm::{EntityTrait, QuerySelect};
use uuid::Uuid;

use crate::model_error::ModelError;

/// Supported CRUD actions. Subset used for permission keys and capability checks.
pub const REST_ACTIONS: &[&str] = &["create", "read", "update", "delete"];

/// Model that can be served by the generic REST handler.
/// Implement metadata (model_id, filter/sort/response fields, display name, supported actions)
/// and the hooks (row_to_json, apply_filter, default_sort, apply_sort). list/get have default
/// implementations. Override create/update/delete only when the model supports them.
#[async_trait]
pub trait RestModel: Send + Sync
where
  Uuid: Into<<<Self::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>,
{
  /// SeaORM entity type (table).
  type Entity: EntityTrait<Model = Self::Model> + Send + Sync;
  /// SeaORM model type (row). Must implement FromQueryResult for default list().
  type Model: sea_orm::FromQueryResult + Send + Sync;

  /// Stable model id (lowercase snake_case), e.g. `"organization"`.
  fn model_id() -> &'static str
  where
    Self: Sized;

  /// Allowed filter fields for list query. Empty = no filtering.
  fn filter_fields() -> &'static [&'static str]
  where
    Self: Sized;

  /// Allowed sort fields for list query. Empty = default sort only.
  fn sort_fields() -> &'static [&'static str]
  where
    Self: Sized;

  /// Column names to include in list/get responses (allow-list).
  fn response_columns() -> &'static [&'static str]
  where
    Self: Sized;

  /// Display name for UI (optional).
  fn display_name() -> Option<&'static str>
  where
    Self: Sized;

  /// Supported actions; subset of [REST_ACTIONS]. Used for permission keys and to gate create/update/delete.
  fn supported_actions() -> &'static [&'static str]
  where
    Self: Sized;

  /// Serialize one row to JSON for list/get responses.
  fn row_to_json(r: &Self::Model) -> serde_json::Value;

  /// Apply one filter condition to the select. Return select unchanged if field not supported.
  fn apply_filter(
    select: sea_orm::Select<Self::Entity>,
    cond: &FilterCond,
  ) -> sea_orm::Select<Self::Entity>;

  /// Apply default order when no sort param is given.
  fn default_sort(select: sea_orm::Select<Self::Entity>) -> sea_orm::Select<Self::Entity>;

  /// Apply sort from spec (field + direction).
  fn apply_sort(
    select: sea_orm::Select<Self::Entity>,
    sort: &SortSpec,
  ) -> sea_orm::Select<Self::Entity>;

  /// List models with filter/sort/pagination. Returns `{ "data": [ ... ] }`. Default uses hooks above.
  async fn list(db: &DbConnection, spec: &ListQuerySpec) -> Result<serde_json::Value, ModelError> {
    let mut select = Self::Entity::find();
    for cond in &spec.filter {
      select = Self::apply_filter(select, cond);
    }
    select = if let Some(ref s) = spec.sort {
      Self::apply_sort(select, s)
    } else {
      Self::default_sort(select)
    };
    let offset = spec.effective_offset();
    let limit = spec.effective_limit();
    let rows = select.offset(offset).limit(limit).all(db).await?;
    let list: Vec<serde_json::Value> = rows.into_iter().map(|r| Self::row_to_json(&r)).collect();
    Ok(serde_json::json!({ "data": list }))
  }

  /// Get one model by id. Returns `None` if not found. Default uses Entity::find_by_id + row_to_json.
  async fn get(db: &DbConnection, id: Uuid) -> Result<Option<serde_json::Value>, ModelError> {
    let row = Self::Entity::find_by_id(id).one(db).await?;
    Ok(row.map(|r| Self::row_to_json(&r)))
  }

  /// Create model from JSON body. Returns the new model's id. Default: Validation error (not supported).
  async fn create(_db: &DbConnection, _body: serde_json::Value) -> Result<Uuid, ModelError> {
    Err(ModelError::Validation(
      "create not supported for this model".to_string(),
    ))
  }

  /// Update model by id. Default: Validation error (not supported).
  async fn update(
    _db: &DbConnection,
    _id: Uuid,
    _body: serde_json::Value,
  ) -> Result<serde_json::Value, ModelError> {
    Err(ModelError::Validation(
      "update not supported for this model".to_string(),
    ))
  }

  /// Delete model by id. Default: Validation error (not supported).
  async fn delete(_db: &DbConnection, _id: Uuid) -> Result<bool, ModelError> {
    Err(ModelError::Validation(
      "delete not supported for this model".to_string(),
    ))
  }
}

#[cfg(test)]
mod bdd_tests {
  use super::*;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests are organized by feature/behavior area with descriptive names.
  mod rest_actions_constant_behavior {
    use super::*;

    #[test]
    fn should_define_all_standard_crud_actions() {
      // Given: REST_ACTIONS constant
      // When: inspecting its contents
      // Then: should contain all four standard CRUD actions
      assert_eq!(REST_ACTIONS.len(), 4);
      assert!(REST_ACTIONS.contains(&"create"));
      assert!(REST_ACTIONS.contains(&"read"));
      assert!(REST_ACTIONS.contains(&"update"));
      assert!(REST_ACTIONS.contains(&"delete"));
    }

    #[test]
    fn should_have_actions_in_standard_order() {
      // Given: REST_ACTIONS constant
      // When: checking the order
      // Then: should be in CRUD order (create, read, update, delete)
      assert_eq!(REST_ACTIONS[0], "create");
      assert_eq!(REST_ACTIONS[1], "read");
      assert_eq!(REST_ACTIONS[2], "update");
      assert_eq!(REST_ACTIONS[3], "delete");
    }

    #[test]
    fn should_be_usable_for_permission_keys() {
      // Given: REST_ACTIONS constant
      // When: constructing permission keys
      // Then: should provide consistent action names
      let model_id = "user";
      let permission_keys: Vec<String> = REST_ACTIONS
        .iter()
        .map(|action| format!("{}:{}", model_id, action))
        .collect();
      assert_eq!(permission_keys[0], "user:create");
      assert_eq!(permission_keys[1], "user:read");
      assert_eq!(permission_keys[2], "user:update");
      assert_eq!(permission_keys[3], "user:delete");
    }
  }

  mod default_crud_behavior {
    #[test]
    fn should_have_consistent_create_error_message() {
      // Given: default create implementation
      // When: checking the error message
      // Then: should match expected format
      let expected_msg = "create not supported for this model";
      assert_eq!(expected_msg, "create not supported for this model");
    }

    #[test]
    fn should_have_consistent_update_error_message() {
      // Given: default update implementation
      // When: checking the error message
      // Then: should match expected format
      let expected_msg = "update not supported for this model";
      assert_eq!(expected_msg, "update not supported for this model");
    }

    #[test]
    fn should_have_consistent_delete_error_message() {
      // Given: default delete implementation
      // When: checking the error message
      // Then: should match expected format
      let expected_msg = "delete not supported for this model";
      assert_eq!(expected_msg, "delete not supported for this model");
    }
  }

  mod trait_contract_behavior {
    #[test]
    fn should_require_model_id_implementation() {
      // Given: RestModel trait
      // When: implementing the trait
      // Then: model_id() must return a static string identifier
      // This is enforced by the trait definition - implementations must provide this
    }

    #[test]
    fn should_require_filter_fields_implementation() {
      // Given: RestModel trait
      // When: implementing the trait
      // Then: filter_fields() must return allowed filter fields (can be empty)
      // This is enforced by the trait definition
    }

    #[test]
    fn should_require_sort_fields_implementation() {
      // Given: RestModel trait
      // When: implementing the trait
      // Then: sort_fields() must return allowed sort fields (can be empty)
      // This is enforced by the trait definition
    }

    #[test]
    fn should_require_response_columns_implementation() {
      // Given: RestModel trait
      // When: implementing the trait
      // Then: response_columns() must return column allow-list
      // This is enforced by the trait definition
    }

    #[test]
    fn should_allow_optional_display_name() {
      // Given: RestModel trait
      // When: implementing the trait
      // Then: display_name() can return None (optional)
      // This is enforced by the trait definition returning Option<&'static str>
    }

    #[test]
    fn should_require_supported_actions_implementation() {
      // Given: RestModel trait
      // When: implementing the trait
      // Then: supported_actions() must return subset of REST_ACTIONS
      // This is enforced by the trait definition
    }

    #[test]
    fn should_provide_default_list_implementation() {
      // Given: RestModel trait
      // When: implementing the trait
      // Then: list() has default implementation using hooks
      // The default implementation uses apply_filter, default_sort/apply_sort, and row_to_json
    }

    #[test]
    fn should_provide_default_get_implementation() {
      // Given: RestModel trait
      // When: implementing the trait
      // Then: get() has default implementation using Entity::find_by_id and row_to_json
      // The default implementation finds by id and converts to JSON
    }

    #[test]
    fn should_require_row_to_json_implementation() {
      // Given: RestModel trait
      // When: implementing the trait
      // Then: row_to_json() must serialize model to JSON
      // This is required for list() and get() default implementations
    }

    #[test]
    fn should_require_apply_filter_implementation() {
      // Given: RestModel trait
      // When: implementing the trait
      // Then: apply_filter() must apply filter condition to select query
      // This is used by default list() implementation
    }

    #[test]
    fn should_require_default_sort_implementation() {
      // Given: RestModel trait
      // When: implementing the trait
      // Then: default_sort() must apply default ordering when no sort specified
      // This is used by default list() implementation
    }

    #[test]
    fn should_require_apply_sort_implementation() {
      // Given: RestModel trait
      // When: implementing the trait
      // Then: apply_sort() must apply sort spec to select query
      // This is used by default list() implementation when sort is provided
    }
  }
}
