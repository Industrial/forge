//! Trait for models served by the generic REST handler.
//! Implement this trait to get list/get/create/update/delete via `/api/entities/{model_id}`.
//! Default implementations are provided for list/get (using hooks below) and for create/update/delete
//! (return "not supported"). Override only what differs.

use async_trait::async_trait;
use forge_db::DbConnection;
use sea_orm::entity::PrimaryKeyTrait;
use sea_orm::{EntityTrait, QuerySelect};
use uuid::Uuid;

use crate::model_error::ModelError;
use crate::query_spec::{FilterCond, ListQuerySpec, SortSpec};

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
