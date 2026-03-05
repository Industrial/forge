//! Generic REST handler per docs/technical-choices/05-single-generic-rest-handler.md.
//! Single route pattern: entity_id + action; 404 unknown entity; authz and scope from context.
//! Epic 6: expand/include rejected with 400; relations as IDs only.
//! Epic 4: list accepts ListQuerySpec (filter, sort, pagination) via query params.

use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use db::auth::Backend;
use forge_auth::token_auth::RequireAuth;
use forge_db::DbConnection;
use serde::Deserialize;
use uuid::Uuid;

use crate::handlers::auth::ScopeFromHeaders;
use crate::handlers::dashboard::require_entity_permission;
use crate::subscriptions::{ChangeEvent, SubscriptionStore};
use crate::query_spec::{
  FilterOperator, ListQuerySpec, SortDirection, DEFAULT_LIMIT, FilterCond, SortSpec,
  validate_filter_cond, validate_offset_limit, validate_sort_field,
};
use crate::registry;
use crate::Error as ForgeError;
use db::models::user;

/// Query params for list: expand/include (rejected), plus filter/sort/pagination (Epic 4).
#[derive(Debug, Deserialize, Default)]
pub struct ListQueryParams {
  #[serde(default)]
  pub expand: Option<String>,
  #[serde(default)]
  pub include: Option<String>,
  #[serde(default)]
  pub filter: Option<String>,
  #[serde(default)]
  pub sort: Option<String>,
  #[serde(default)]
  pub order: Option<String>,
  #[serde(default)]
  pub offset: Option<u64>,
  #[serde(default)]
  pub limit: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct RawFilterCond {
  field: String,
  operator: String,
  value: Option<serde_json::Value>,
}

/// Parse list query params into ListQuerySpec. Returns Err(message) for invalid filter/sort/offset/limit.
/// Public so dashboard (and other legacy list handlers) can adopt ListQuerySpec.
pub fn parse_list_query_spec(params: &ListQueryParams) -> Result<ListQuerySpec, String> {
  let mut spec = ListQuerySpec::default();

  if let Some(ref s) = params.filter {
    if !s.trim().is_empty() {
      let raw: Vec<RawFilterCond> =
        serde_json::from_str(s).map_err(|e| format!("invalid filter JSON: {}", e))?;
      for r in raw {
        let op = FilterOperator::from_str(&r.operator)
          .ok_or_else(|| format!("invalid filter operator: {}", r.operator))?;
        spec.filter.push(FilterCond {
          field: r.field,
          operator: op,
          value: r.value,
        });
      }
    }
  }

  if let Some(ref field) = params.sort {
    if !field.trim().is_empty() {
      let direction = params
        .order
        .as_deref()
        .and_then(SortDirection::from_str)
        .unwrap_or(SortDirection::Asc);
      spec.sort = Some(SortSpec {
        field: field.clone(),
        direction,
      });
    }
  }

  let offset = params.offset.unwrap_or(0);
  let limit = params.limit.unwrap_or(DEFAULT_LIMIT);
  spec.offset_limit = Some(validate_offset_limit(offset, limit)?);

  Ok(spec)
}

fn reject_expand_include(params: &ListQueryParams) -> Option<axum::response::Response> {
  if params.expand.is_some() || params.include.is_some() {
    Some(
      (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({
          "error": "Bad Request",
          "message": "expand and include are not supported; use separate list/get requests for related data"
        })),
      )
        .into_response(),
    )
  } else {
    None
  }
}

/// GET /api/entities/:entity_id — list entities. Query spec (filter, sort, pagination) via query params.
/// 404 unknown entity_id; 403 missing entity.read; scope from headers. Rejects expand/include (400).
/// Query params: filter (JSON array), sort, order, offset, limit. Validates filter/sort against entity allowed fields (400).
pub async fn list_entities(
  Path(entity_id): Path<String>,
  Query(params): Query<ListQueryParams>,
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = reject_expand_include(&params) {
    return Ok(resp);
  }
  if !registry::is_known_model(entity_id.as_str()) {
    return Ok((
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({
        "error": "Not Found",
        "message": "Unknown entity"
      })),
    )
      .into_response());
  }
  let user = &auth.0;
  if let Some(resp) =
    require_entity_permission(user, &db, Some(&scope), entity_id.as_str(), "read").await
  {
    return Ok(resp);
  }
  let spec = match parse_list_query_spec(&params) {
    Ok(s) => s,
    Err(msg) => {
      return Ok((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": "Bad Request", "message": msg })),
      )
        .into_response());
    }
  };
  let allowed_filter = registry::effective_filter_fields(entity_id.as_str());
  let allowed_sort = registry::effective_sort_fields(entity_id.as_str());
  for cond in &spec.filter {
    if let Err(msg) = validate_filter_cond(cond, allowed_filter) {
      return Ok((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": "Bad Request", "message": msg })),
      )
        .into_response());
    }
  }
  if let Some(ref sort) = spec.sort {
    if let Err(msg) = validate_sort_field(&sort.field, allowed_sort) {
      return Ok((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": "Bad Request", "message": msg })),
      )
        .into_response());
    }
  }
  match registry::list_models(entity_id.as_str(), &db, &spec)
    .await
    .map_err(crate::Error::from)
  {
    Ok(v) => Ok(Json(v).into_response()),
    Err(ForgeError::Auth(StatusCode::NOT_FOUND, _)) => Ok((
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({
        "error": "Not Found",
        "message": "Unknown entity"
      })),
    )
      .into_response()),
    Err(e) => Err(e),
  }
}

/// List entity by id with query spec (filter, sort, pagination). Validates spec against entity allowed fields.
/// Returns the list result payload (e.g. `{ "data": [...] }`) for use by REST or RPC.
pub async fn list_entity_with_spec(
  db: &DbConnection,
  entity_id: &str,
  spec: &ListQuerySpec,
) -> Result<serde_json::Value, ForgeError> {
  let allowed_filter = registry::effective_filter_fields(entity_id);
  let allowed_sort = registry::effective_sort_fields(entity_id);
  for cond in &spec.filter {
    validate_filter_cond(cond, allowed_filter)
      .map_err(|msg| ForgeError::Auth(StatusCode::BAD_REQUEST, msg))?;
  }
  if let Some(ref sort) = spec.sort {
    validate_sort_field(&sort.field, allowed_sort)
      .map_err(|msg| ForgeError::Auth(StatusCode::BAD_REQUEST, msg))?;
  }
  registry::list_models(entity_id, db, spec)
    .await
    .map_err(crate::Error::from)
}

/// GET /api/entities/:entity_id/:id — get one entity by id. 404 unknown entity or not found. Rejects expand/include (400).
pub async fn get_entity_by_id(
  Path((entity_id, id_str)): Path<(String, String)>,
  Query(params): Query<ListQueryParams>,
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = reject_expand_include(&params) {
    return Ok(resp);
  }
  if !registry::is_known_model(entity_id.as_str()) {
    return Ok((
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({
        "error": "Not Found",
        "message": "Unknown entity"
      })),
    )
      .into_response());
  }
  let user = &auth.0;
  if let Some(resp) =
    require_entity_permission(user, &db, Some(&scope), entity_id.as_str(), "read").await
  {
    return Ok(resp);
  }
  let id = match Uuid::parse_str(&id_str) {
    Ok(u) => u,
    Err(_) => {
      return Ok((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": "Bad Request", "message": "Invalid id" })),
      )
        .into_response());
    }
  };
  match registry::get_model(entity_id.as_str(), &db, id)
    .await
    .map_err(crate::Error::from)
  {
    Ok(Some(v)) => Ok(Json(v).into_response()),
    Ok(None) => Ok((
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({
        "error": "Not Found",
        "message": "Resource not found"
      })),
    )
      .into_response()),
    Err(ForgeError::Auth(StatusCode::NOT_FOUND, _)) => Ok((
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({
        "error": "Not Found",
        "message": "Unknown entity"
      })),
    )
      .into_response()),
    Err(e) => Err(e),
  }
}

/// Reject non-object body for create/update; returns 400 response if invalid.
fn require_object_body(body: &serde_json::Value) -> Option<axum::response::Response> {
  if !body.is_object() {
    Some(
      (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({
          "error": "Bad Request",
          "message": "Request body must be a JSON object"
        })),
      )
        .into_response(),
    )
  } else {
    None
  }
}

/// POST /api/entities/:entity_id — create entity. 404 unknown entity; 403 missing entity.create.
pub async fn create_entity(
  Path(entity_id): Path<String>,
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(subscriptions): Extension<SubscriptionStore>,
  Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, ForgeError> {
  if !registry::is_known_model(entity_id.as_str()) {
    return Ok((
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({
        "error": "Not Found",
        "message": "Unknown entity"
      })),
    )
      .into_response());
  }
  let user = &auth.0;
  if let Some(resp) =
    require_entity_permission(user, &db, Some(&scope), entity_id.as_str(), "create").await
  {
    return Ok(resp);
  }
  if let Some(resp) = require_object_body(&body) {
    return Ok(resp);
  }
  match registry::create_model(entity_id.as_str(), &db, body)
    .await
    .map_err(crate::Error::from)
  {
    Ok(id) => {
      subscriptions.publish_change(ChangeEvent {
        model_id: entity_id.clone(),
        resource_id: id,
        action: "create".to_string(),
        organization_id: Some(scope.organization_id),
      });
      let body = registry::get_model(entity_id.as_str(), &db, id)
        .await
        .map_err(crate::Error::from)?
        .ok_or_else(|| ForgeError::Generic("created resource not found".to_string()))?;
      Ok((
        StatusCode::CREATED,
        Json(body),
      )
        .into_response())
    }
    Err(ForgeError::Auth(StatusCode::NOT_FOUND, _)) => Ok((
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({
        "error": "Not Found",
        "message": "Unknown entity"
      })),
    )
      .into_response()),
    Err(ForgeError::Generic(msg)) => Ok((
      StatusCode::UNPROCESSABLE_ENTITY,
      Json(serde_json::json!({ "error": "Unprocessable Entity", "message": msg })),
    )
      .into_response()),
    Err(e) => Err(e),
  }
}

/// PATCH /api/entities/:entity_id/:id — update entity. 404 unknown entity or not found; 403 missing entity.update.
pub async fn update_entity(
  Path((entity_id, id_str)): Path<(String, String)>,
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(subscriptions): Extension<SubscriptionStore>,
  Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, ForgeError> {
  if !registry::is_known_model(entity_id.as_str()) {
    return Ok((
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({
        "error": "Not Found",
        "message": "Unknown entity"
      })),
    )
      .into_response());
  }
  let user = &auth.0;
  if let Some(resp) =
    require_entity_permission(user, &db, Some(&scope), entity_id.as_str(), "update").await
  {
    return Ok(resp);
  }
  let id = match Uuid::parse_str(&id_str) {
    Ok(u) => u,
    Err(_) => {
      return Ok((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": "Bad Request", "message": "Invalid id" })),
      )
        .into_response());
    }
  };
  if let Some(resp) = require_object_body(&body) {
    return Ok(resp);
  }
  match registry::update_model(entity_id.as_str(), &db, id, body)
    .await
    .map_err(crate::Error::from)
  {
    Ok(updated) => {
      subscriptions.publish_change(ChangeEvent {
        model_id: entity_id.clone(),
        resource_id: id,
        action: "update".to_string(),
        organization_id: Some(scope.organization_id),
      });
      Ok(Json(updated).into_response())
    }
    Err(ForgeError::Auth(StatusCode::NOT_FOUND, _)) => Ok((
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({
        "error": "Not Found",
        "message": "Unknown entity"
      })),
    )
      .into_response()),
    Err(ForgeError::Generic(msg)) if msg.contains("not found") => Ok((
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({ "error": "Not Found", "message": msg })),
    )
      .into_response()),
    Err(e) => Err(e),
  }
}

/// DELETE /api/entities/:entity_id/:id — delete entity. 404 unknown entity or not found; 403 missing entity.delete.
pub async fn delete_entity(
  Path((entity_id, id_str)): Path<(String, String)>,
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(subscriptions): Extension<SubscriptionStore>,
) -> Result<impl IntoResponse, ForgeError> {
  if !registry::is_known_model(entity_id.as_str()) {
    return Ok((
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({
        "error": "Not Found",
        "message": "Unknown entity"
      })),
    )
      .into_response());
  }
  let user = &auth.0;
  if let Some(resp) =
    require_entity_permission(user, &db, Some(&scope), entity_id.as_str(), "delete").await
  {
    return Ok(resp);
  }
  let id = match Uuid::parse_str(&id_str) {
    Ok(u) => u,
    Err(_) => {
      return Ok((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": "Bad Request", "message": "Invalid id" })),
      )
        .into_response());
    }
  };
  match registry::delete_model(entity_id.as_str(), &db, id)
    .await
    .map_err(crate::Error::from)
  {
    Ok(true) => {
      subscriptions.publish_change(ChangeEvent {
        model_id: entity_id.clone(),
        resource_id: id,
        action: "delete".to_string(),
        organization_id: Some(scope.organization_id),
      });
      Ok(Json(serde_json::json!({ "ok": true })).into_response())
    }
    Ok(false) => Ok((
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({ "error": "Not Found", "message": "Resource not found" })),
    )
      .into_response()),
    Err(ForgeError::Auth(StatusCode::NOT_FOUND, _)) => Ok((
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({
        "error": "Not Found",
        "message": "Unknown entity"
      })),
    )
      .into_response()),
    Err(e) => Err(e),
  }
}

