//! Generic REST handler per docs/technical-choices/05-single-generic-rest-handler.md.
//! Single route pattern: entity_id + action; 404 unknown entity; authz and scope from context.
//! Epic 6: expand/include rejected with 400; relations as IDs only.
//! Epic 4: list accepts ListQuerySpec (filter, sort, pagination) via query params.

use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use db::auth::Backend;
use forge_auth::token_auth::RequireAuth;
use forge_db::DbConnection;
use serde::Deserialize;
use uuid::Uuid;

use crate::Error as ForgeError;
use crate::handlers::auth::ScopeFromHeaders;
use crate::handlers::dashboard::require_entity_permission;
use crate::query_spec::{
  DEFAULT_LIMIT, FilterCond, FilterOperator, ListQuerySpec, SortDirection, SortSpec,
  validate_filter_cond, validate_offset_limit, validate_sort_field,
};
use crate::registry;
use db::models::user;
use forge_live::{ChangeEvent, SubscriptionStore};

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
        let op = FilterOperator::try_parse(&r.operator)
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
        .and_then(SortDirection::try_parse)
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
  tracing::debug!(
    target: "app::handlers::generic_entity",
    entity_id = %entity_id,
    filter = ?params.filter.as_deref(),
    sort = ?params.sort.as_deref(),
    offset = ?params.offset,
    limit = ?params.limit,
    "list_entities"
  );
  if let Some(resp) = reject_expand_include(&params) {
    return Ok(resp);
  }
  if !registry::is_known_model(entity_id.as_str()) {
    return Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
          "error": "Not Found",
          "message": "Unknown entity"
        })),
      )
        .into_response(),
    );
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
      return Ok(
        (
          StatusCode::BAD_REQUEST,
          Json(serde_json::json!({ "error": "Bad Request", "message": msg })),
        )
          .into_response(),
      );
    }
  };
  let allowed_filter = registry::effective_filter_fields(entity_id.as_str());
  let allowed_sort = registry::effective_sort_fields(entity_id.as_str());
  for cond in &spec.filter {
    if let Err(msg) = validate_filter_cond(cond, allowed_filter) {
      return Ok(
        (
          StatusCode::BAD_REQUEST,
          Json(serde_json::json!({ "error": "Bad Request", "message": msg })),
        )
          .into_response(),
      );
    }
  }
  if let Some(ref sort) = spec.sort {
    if let Err(msg) = validate_sort_field(&sort.field, allowed_sort) {
      return Ok(
        (
          StatusCode::BAD_REQUEST,
          Json(serde_json::json!({ "error": "Bad Request", "message": msg })),
        )
          .into_response(),
      );
    }
  }
  match registry::list_models(entity_id.as_str(), &db, &spec)
    .await
    .map_err(crate::Error::from)
  {
    Ok(v) => Ok(Json(v).into_response()),
    Err(ForgeError::Auth(StatusCode::NOT_FOUND, _)) => Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
          "error": "Not Found",
          "message": "Unknown entity"
        })),
      )
        .into_response(),
    ),
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
  tracing::debug!(
    target: "app::handlers::generic_entity",
    entity_id = %entity_id,
    id = %id_str,
    "get_entity_by_id"
  );
  if let Some(resp) = reject_expand_include(&params) {
    return Ok(resp);
  }
  if !registry::is_known_model(entity_id.as_str()) {
    return Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
          "error": "Not Found",
          "message": "Unknown entity"
        })),
      )
        .into_response(),
    );
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
      return Ok(
        (
          StatusCode::BAD_REQUEST,
          Json(serde_json::json!({ "error": "Bad Request", "message": "Invalid id" })),
        )
          .into_response(),
      );
    }
  };
  match registry::get_model(entity_id.as_str(), &db, id)
    .await
    .map_err(crate::Error::from)
  {
    Ok(Some(v)) => Ok(Json(v).into_response()),
    Ok(None) => Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
          "error": "Not Found",
          "message": "Resource not found"
        })),
      )
        .into_response(),
    ),
    Err(ForgeError::Auth(StatusCode::NOT_FOUND, _)) => Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
          "error": "Not Found",
          "message": "Unknown entity"
        })),
      )
        .into_response(),
    ),
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
  tracing::debug!(
    target: "app::handlers::generic_entity",
    entity_id = %entity_id,
    "create_entity"
  );
  if !registry::is_known_model(entity_id.as_str()) {
    return Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
          "error": "Not Found",
          "message": "Unknown entity"
        })),
      )
        .into_response(),
    );
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
      Ok((StatusCode::CREATED, Json(body)).into_response())
    }
    Err(ForgeError::Auth(StatusCode::NOT_FOUND, _)) => Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
          "error": "Not Found",
          "message": "Unknown entity"
        })),
      )
        .into_response(),
    ),
    Err(ForgeError::Generic(msg)) => Ok(
      (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": "Unprocessable Entity", "message": msg })),
      )
        .into_response(),
    ),
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
  tracing::debug!(
    target: "app::handlers::generic_entity",
    entity_id = %entity_id,
    id = %id_str,
    "update_entity"
  );
  if !registry::is_known_model(entity_id.as_str()) {
    return Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
          "error": "Not Found",
          "message": "Unknown entity"
        })),
      )
        .into_response(),
    );
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
      return Ok(
        (
          StatusCode::BAD_REQUEST,
          Json(serde_json::json!({ "error": "Bad Request", "message": "Invalid id" })),
        )
          .into_response(),
      );
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
    Err(ForgeError::Auth(StatusCode::NOT_FOUND, _)) => Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
          "error": "Not Found",
          "message": "Unknown entity"
        })),
      )
        .into_response(),
    ),
    Err(ForgeError::Generic(msg)) if msg.contains("not found") => Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Not Found", "message": msg })),
      )
        .into_response(),
    ),
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
  tracing::debug!(
    target: "app::handlers::generic_entity",
    entity_id = %entity_id,
    id = %id_str,
    "delete_entity"
  );
  if !registry::is_known_model(entity_id.as_str()) {
    return Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
          "error": "Not Found",
          "message": "Unknown entity"
        })),
      )
        .into_response(),
    );
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
      return Ok(
        (
          StatusCode::BAD_REQUEST,
          Json(serde_json::json!({ "error": "Bad Request", "message": "Invalid id" })),
        )
          .into_response(),
      );
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
    Ok(false) => Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Not Found", "message": "Resource not found" })),
      )
        .into_response(),
    ),
    Err(ForgeError::Auth(StatusCode::NOT_FOUND, _)) => Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
          "error": "Not Found",
          "message": "Unknown entity"
        })),
      )
        .into_response(),
    ),
    Err(e) => Err(e),
  }
}

#[cfg(test)]
mod bdd_tests {
  use super::*;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests verify generic entity handler behaviors for CRUD operations.

  mod query_parsing_behavior {
    use super::*;

    #[test]
    fn should_parse_empty_query_params_into_default_spec() {
      // Given: empty query parameters
      let params = ListQueryParams::default();

      // When: parsing query spec
      let spec = parse_list_query_spec(&params).unwrap();

      // Then: should return default spec with no filters, no sort, default pagination
      assert!(spec.filter.is_empty());
      assert!(spec.sort.is_none());
      assert_eq!(spec.offset_limit.unwrap().offset, 0);
      assert_eq!(spec.offset_limit.unwrap().limit, DEFAULT_LIMIT);
    }

    #[test]
    fn should_parse_filter_query_param() {
      // Given: query params with filter JSON
      let params = ListQueryParams {
        filter: Some(
          r#"[{"field": "name", "operator": "eq", "value": "test"}]"#.to_string(),
        ),
        ..Default::default()
      };

      // When: parsing query spec
      let spec = parse_list_query_spec(&params).unwrap();

      // Then: should parse filter conditions
      assert_eq!(spec.filter.len(), 1);
      assert_eq!(spec.filter[0].field, "name");
      assert_eq!(spec.filter[0].operator, FilterOperator::Eq);
    }

    #[test]
    fn should_parse_sort_query_params() {
      // Given: query params with sort and order
      let params = ListQueryParams {
        sort: Some("name".to_string()),
        order: Some("desc".to_string()),
        ..Default::default()
      };

      // When: parsing query spec
      let spec = parse_list_query_spec(&params).unwrap();

      // Then: should parse sort specification
      assert!(spec.sort.is_some());
      let sort = spec.sort.unwrap();
      assert_eq!(sort.field, "name");
      assert_eq!(sort.direction, SortDirection::Desc);
    }

    #[test]
    fn should_parse_pagination_params() {
      // Given: query params with offset and limit
      let params = ListQueryParams {
        offset: Some(10),
        limit: Some(20),
        ..Default::default()
      };

      // When: parsing query spec
      let spec = parse_list_query_spec(&params).unwrap();

      // Then: should parse pagination
      let offset_limit = spec.offset_limit.unwrap();
      assert_eq!(offset_limit.offset, 10);
      assert_eq!(offset_limit.limit, 20);
    }

    #[test]
    fn should_reject_invalid_filter_json() {
      // Given: query params with invalid filter JSON
      let params = ListQueryParams {
        filter: Some("invalid json".to_string()),
        ..Default::default()
      };

      // When: parsing query spec
      let result = parse_list_query_spec(&params);

      // Then: should return error about invalid JSON
      assert!(result.is_err());
      let err = result.unwrap_err();
      assert!(err.contains("invalid filter JSON"));
    }

    #[test]
    fn should_reject_invalid_filter_operator() {
      // Given: query params with invalid filter operator
      let params = ListQueryParams {
        filter: Some(
          r#"[{"field": "name", "operator": "invalid_op", "value": "test"}]"#.to_string(),
        ),
        ..Default::default()
      };

      // When: parsing query spec
      let result = parse_list_query_spec(&params);

      // Then: should return error about invalid operator
      assert!(result.is_err());
      let err = result.unwrap_err();
      assert!(err.contains("invalid filter operator"));
    }
  }

  mod expand_include_rejection_behavior {
    use super::*;

    #[test]
    fn should_reject_expand_parameter() {
      // Given: query params with expand parameter
      let params = ListQueryParams {
        expand: Some("relation".to_string()),
        ..Default::default()
      };

      // When: checking for expand/include
      let response = reject_expand_include(&params);

      // Then: should return 400 Bad Request response
      assert!(response.is_some());
    }

    #[test]
    fn should_reject_include_parameter() {
      // Given: query params with include parameter
      let params = ListQueryParams {
        include: Some("relation".to_string()),
        ..Default::default()
      };

      // When: checking for expand/include
      let response = reject_expand_include(&params);

      // Then: should return 400 Bad Request response
      assert!(response.is_some());
    }

    #[test]
    fn should_accept_params_without_expand_or_include() {
      // Given: query params without expand or include
      let params = ListQueryParams {
        filter: Some(r#"[{"field": "name", "operator": "eq", "value": "test"}]"#.to_string()),
        ..Default::default()
      };

      // When: checking for expand/include
      let response = reject_expand_include(&params);

      // Then: should return None (no rejection)
      assert!(response.is_none());
    }
  }

  mod body_validation_behavior {
    use super::*;

    #[test]
    fn should_reject_non_object_body() {
      // Given: a JSON array body (not an object)
      let body = serde_json::json!([1, 2, 3]);

      // When: validating body
      let response = require_object_body(&body);

      // Then: should return 400 Bad Request response
      assert!(response.is_some());
    }

    #[test]
    fn should_reject_string_body() {
      // Given: a JSON string body (not an object)
      let body = serde_json::json!("not an object");

      // When: validating body
      let response = require_object_body(&body);

      // Then: should return 400 Bad Request response
      assert!(response.is_some());
    }

    #[test]
    fn should_accept_object_body() {
      // Given: a JSON object body
      let body = serde_json::json!({"name": "test", "value": 123});

      // When: validating body
      let response = require_object_body(&body);

      // Then: should return None (validation passes)
      assert!(response.is_none());
    }

    #[test]
    fn should_accept_empty_object_body() {
      // Given: an empty JSON object body
      let body = serde_json::json!({});

      // When: validating body
      let response = require_object_body(&body);

      // Then: should return None (validation passes)
      assert!(response.is_none());
    }
  }

  mod id_validation_behavior {
    use super::*;

    #[test]
    fn should_validate_uuid_format_for_id() {
      // Given: a valid UUID string
      let valid_uuid = uuid::Uuid::new_v4().to_string();

      // When: parsing UUID
      let result = Uuid::parse_str(&valid_uuid);

      // Then: should parse successfully
      assert!(result.is_ok());
    }

    #[test]
    fn should_reject_invalid_uuid_format() {
      // Given: an invalid UUID string
      let invalid_uuid = "not-a-uuid";

      // When: parsing UUID
      let result = Uuid::parse_str(invalid_uuid);

      // Then: should return error
      assert!(result.is_err());
    }
  }
}
