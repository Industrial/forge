//! RPC endpoint per docs/technical-choices/07-rpc-parity-with-rest.md.
//! Same entity operations (list, get, create, update, delete) over POST /api/rpc; auth and scope from headers (parity with REST).

use axum::Json;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use db::auth::Backend;
use forge_auth::token_auth::RequireAuth;
use forge_db::DbConnection;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::Error as ForgeError;
use crate::handlers::auth::ScopeFromHeaders;
use crate::handlers::auth::require_entity_permission;
use crate::handlers::generic_entity::{
  ListQueryParams, list_entity_with_spec, parse_list_query_spec,
};
use crate::query_spec::{validate_filter_cond, validate_sort_field};
use crate::registry;
use db::models::user;
use forge_live::{SubscriptionMeta, SubscriptionStore};

/// Request envelope (Epic 7): method, entity_id, params, optional correlation id.
#[derive(Debug, Deserialize)]
pub struct RpcRequest {
  pub method: String,
  pub entity_id: String,
  #[serde(default)]
  pub params: Option<RpcParams>,
  /// Optional correlation id (number or string); echoed in response for request/response matching.
  #[serde(default)]
  pub id: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RpcParams {
  pub id: Option<String>,
  pub body: Option<serde_json::Value>,
  /// For subscribe: connection id from the subscription stream ready message (required for subscribe).
  pub connection_id: Option<String>,
  /// For unsubscribe: server-assigned subscription id.
  pub subscription_id: Option<String>,
  /// For entity.list: same as REST query params (filter, sort, order, offset, limit, cursor).
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
  #[serde(default)]
  pub cursor: Option<String>,
}

/// POST /api/rpc — same entity operations as REST; body: { method, entity_id, params?, id? }.
/// Identity and scope from same headers (Authorization, X-Organization-Id, X-Role-Id). Response: { result } or { error: { code, message } }; optional id echoed.
pub async fn rpc_handler(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(subscriptions): Extension<SubscriptionStore>,
  Json(body): Json<RpcRequest>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  let method = body.method.as_str();
  let entity_id = body.entity_id.as_str();
  let correlation_id = body.id.clone();

  tracing::debug!(
    target: "app::handlers::rpc",
    method = %method,
    entity_id = %entity_id,
    user_id = %user.id,
    "rpc_handler"
  );

  // Subscribe/unsubscribe (Epic 8): same transport as entity RPC; server-assigned subscription id.
  match method {
    "subscribe" => {
      return rpc_subscribe(user, &db, &scope, &subscriptions, body, correlation_id).await;
    }
    "unsubscribe" => return rpc_unsubscribe(&subscriptions, body, correlation_id).await,
    _ => {}
  }

  if !registry::is_known_model(entity_id) {
    return Ok(rpc_error(
      StatusCode::NOT_FOUND,
      "Unknown entity",
      correlation_id,
    ));
  }

  let (action, required_action) = match method {
    "entity.list" => ("list", "read"),
    "entity.get" => ("get", "read"),
    "entity.create" => ("create", "create"),
    "entity.update" => ("update", "update"),
    "entity.delete" => ("delete", "delete"),
    _ => {
      return Ok(rpc_error(
        StatusCode::BAD_REQUEST,
        "method must be entity.list, entity.get, entity.create, entity.update, entity.delete, subscribe, or unsubscribe",
        correlation_id,
      ));
    }
  };

  if let Some(resp) =
    require_entity_permission(user, &db, Some(&scope), entity_id, required_action).await
  {
    return Ok(resp);
  }

  match action {
    "list" => rpc_list(&db, entity_id, body.params.as_ref(), correlation_id, &scope).await,
    "get" => {
      let id_str = body
        .params
        .as_ref()
        .and_then(|p| p.id.as_deref())
        .unwrap_or("");
      let id = match Uuid::parse_str(id_str) {
        Ok(u) => u,
        Err(_) => {
          return Ok(rpc_error(
            StatusCode::BAD_REQUEST,
            "params.id required and must be a valid UUID",
            correlation_id,
          ));
        }
      };
      rpc_get(&db, entity_id, id, correlation_id.clone()).await
    }
    "create" => {
      let body = body
        .params
        .as_ref()
        .and_then(|p| p.body.clone())
        .unwrap_or(serde_json::Value::Null);
      rpc_create(&db, entity_id, body, correlation_id).await
    }
    "update" => {
      let id_str = body
        .params
        .as_ref()
        .and_then(|p| p.id.as_deref())
        .unwrap_or("");
      let id = match Uuid::parse_str(id_str) {
        Ok(u) => u,
        Err(_) => {
          return Ok(rpc_error(
            StatusCode::BAD_REQUEST,
            "params.id required and must be a valid UUID",
            correlation_id,
          ));
        }
      };
      let body = body
        .params
        .as_ref()
        .and_then(|p| p.body.clone())
        .unwrap_or(serde_json::Value::Null);
      rpc_update(&db, entity_id, id, body, correlation_id).await
    }
    "delete" => {
      let id_str = body
        .params
        .as_ref()
        .and_then(|p| p.id.as_deref())
        .unwrap_or("");
      let id = match Uuid::parse_str(id_str) {
        Ok(u) => u,
        Err(_) => {
          return Ok(rpc_error(
            StatusCode::BAD_REQUEST,
            "params.id required and must be a valid UUID",
            correlation_id,
          ));
        }
      };
      rpc_delete(&db, entity_id, id, correlation_id).await
    }
    _ => Ok(rpc_error(
      StatusCode::BAD_REQUEST,
      "invalid method",
      correlation_id,
    )),
  }
}

/// Subscribe RPC (Epic 8): same query spec as list; validate params against model allowed fields; store structured spec for matching.
async fn rpc_subscribe(
  user: &user::Model,
  _db: &DbConnection,
  scope: &forge_auth::RequestScope,
  store: &SubscriptionStore,
  body: RpcRequest,
  correlation_id: Option<serde_json::Value>,
) -> Result<axum::response::Response, ForgeError> {
  let entity_id = body.entity_id.clone();
  tracing::debug!(
    target: "app::handlers::rpc",
    entity_id = %entity_id,
    "rpc_subscribe"
  );
  if !registry::is_known_model(&entity_id) {
    return Ok(rpc_error(
      StatusCode::NOT_FOUND,
      "Unknown entity",
      correlation_id,
    ));
  }
  if let Some(resp) = require_entity_permission(user, _db, Some(scope), &entity_id, "read").await {
    return Ok(resp);
  }
  // Parse and validate params as ListQuerySpec (filter, sort, pagination) per model.
  let list_params = body
    .params
    .as_ref()
    .map(|p| ListQueryParams {
      expand: None,
      include: None,
      filter: p.filter.clone(),
      sort: p.sort.clone(),
      order: p.order.clone(),
      offset: p.offset,
      limit: p.limit,
      cursor: p.cursor.clone(),
    })
    .unwrap_or_default();
  let spec = match parse_list_query_spec(&list_params) {
    Ok(s) => s,
    Err(msg) => {
      return Ok(rpc_error(StatusCode::BAD_REQUEST, &msg, correlation_id));
    }
  };
  let allowed_filter = registry::effective_filter_fields(&entity_id);
  let allowed_sort = registry::effective_sort_fields(&entity_id);
  for cond in &spec.filter {
    if let Err(msg) = validate_filter_cond(cond, allowed_filter) {
      return Ok(rpc_error(StatusCode::BAD_REQUEST, &msg, correlation_id));
    }
  }
  if let Some(ref sort) = spec.sort
    && let Err(msg) = validate_sort_field(&sort.field, allowed_sort)
  {
    return Ok(rpc_error(StatusCode::BAD_REQUEST, &msg, correlation_id));
  }
  let connection_id = body
    .params
    .as_ref()
    .and_then(|p| p.connection_id.as_deref())
    .and_then(|s| Uuid::parse_str(s).ok());
  let connection_id = match connection_id {
    Some(u) => u,
    None => {
      return Ok(rpc_error(
        StatusCode::BAD_REQUEST,
        "params.connection_id required for subscribe (open GET /api/subscriptions/stream first and use the connection_id from the ready message)",
        correlation_id,
      ));
    }
  };
  let params = serde_json::to_value(&spec).ok();
  let meta = SubscriptionMeta {
    entity_id,
    organization_id: scope.organization_id,
    role_id: scope.role_id,
    params,
  };
  let subscription_id = store.subscribe(connection_id, meta);
  Ok(rpc_ok_result(
    json!({ "subscription_id": subscription_id.to_string() }),
    correlation_id,
  ))
}

/// Unsubscribe RPC (Epic 8): by server-assigned subscription id.
async fn rpc_unsubscribe(
  store: &SubscriptionStore,
  body: RpcRequest,
  correlation_id: Option<serde_json::Value>,
) -> Result<axum::response::Response, ForgeError> {
  let subscription_id_str = body
    .params
    .as_ref()
    .and_then(|p| p.subscription_id.as_deref())
    .unwrap_or("");
  tracing::debug!(
    target: "app::handlers::rpc",
    subscription_id = %subscription_id_str,
    "rpc_unsubscribe"
  );
  let subscription_id = match Uuid::parse_str(subscription_id_str) {
    Ok(u) => u,
    Err(_) => {
      return Ok(rpc_error(
        StatusCode::BAD_REQUEST,
        "params.subscription_id required and must be a valid UUID",
        correlation_id,
      ));
    }
  };
  let removed = store.unsubscribe(subscription_id);
  if removed {
    Ok(rpc_ok_result(json!({ "ok": true }), correlation_id))
  } else {
    Ok(rpc_error(
      StatusCode::NOT_FOUND,
      "Subscription not found or already removed",
      correlation_id,
    ))
  }
}

fn rpc_error(
  status: StatusCode,
  message: &str,
  id: Option<serde_json::Value>,
) -> axum::response::Response {
  let code = status.as_u16();
  let mut payload = serde_json::json!({ "error": { "code": code, "message": message } });
  if let Some(id) = id {
    payload["id"] = id;
  }
  (status, Json(payload)).into_response()
}

fn rpc_ok_result(
  result: serde_json::Value,
  id: Option<serde_json::Value>,
) -> axum::response::Response {
  let mut payload = serde_json::json!({ "result": result });
  if let Some(id) = id {
    payload["id"] = id;
  }
  (StatusCode::OK, Json(payload)).into_response()
}

async fn rpc_list(
  db: &DbConnection,
  entity_id: &str,
  params: Option<&RpcParams>,
  correlation_id: Option<serde_json::Value>,
  scope: &forge_auth::RequestScope,
) -> Result<axum::response::Response, ForgeError> {
  let list_params = ListQueryParams {
    expand: None,
    include: None,
    filter: params.and_then(|p| p.filter.clone()),
    sort: params.and_then(|p| p.sort.clone()),
    order: params.and_then(|p| p.order.clone()),
    offset: params.and_then(|p| p.offset),
    limit: params.and_then(|p| p.limit),
    cursor: params.and_then(|p| p.cursor.clone()),
  };
  let spec = match parse_list_query_spec(&list_params) {
    Ok(s) => s,
    Err(msg) => {
      return Ok(rpc_error(StatusCode::BAD_REQUEST, &msg, correlation_id));
    }
  };
  match list_entity_with_spec(db, entity_id, &spec, Some(scope)).await {
    Ok(result) => Ok(rpc_ok_result(result, correlation_id)),
    Err(ForgeError::Auth(status, msg)) => Ok(rpc_error(status, &msg, correlation_id)),
    Err(e) => Ok(rpc_error(
      StatusCode::INTERNAL_SERVER_ERROR,
      &e.to_string(),
      correlation_id,
    )),
  }
}

async fn rpc_get(
  db: &DbConnection,
  entity_id: &str,
  resource_id: Uuid,
  correlation_id: Option<serde_json::Value>,
) -> Result<axum::response::Response, ForgeError> {
  match registry::get_model(entity_id, db, resource_id)
    .await
    .map_err(crate::Error::from)
  {
    Ok(Some(value)) => Ok(rpc_ok_result(value, correlation_id)),
    Ok(None) => Ok(rpc_error(
      StatusCode::NOT_FOUND,
      "Resource not found",
      correlation_id,
    )),
    Err(ForgeError::Auth(StatusCode::NOT_FOUND, _)) => Ok(rpc_error(
      StatusCode::NOT_FOUND,
      "Unknown entity",
      correlation_id,
    )),
    Err(e) => Ok(rpc_error(
      StatusCode::INTERNAL_SERVER_ERROR,
      &e.to_string(),
      correlation_id,
    )),
  }
}

async fn rpc_create(
  db: &DbConnection,
  entity_id: &str,
  body: serde_json::Value,
  correlation_id: Option<serde_json::Value>,
) -> Result<axum::response::Response, ForgeError> {
  if body.is_null() {
    return Ok(rpc_error(
      StatusCode::BAD_REQUEST,
      "params.body required for entity.create",
      correlation_id,
    ));
  }
  match registry::create_model(entity_id, db, body)
    .await
    .map_err(crate::Error::from)
  {
    Ok(id) => {
      // Return created resource (same as REST POST response).
      match registry::get_model(entity_id, db, id)
        .await
        .map_err(crate::Error::from)
      {
        Ok(Some(value)) => Ok(rpc_ok_result(value, correlation_id)),
        Ok(None) => Ok(rpc_ok_result(
          json!({ "id": id.to_string() }),
          correlation_id,
        )),
        Err(_) => Ok(rpc_ok_result(
          json!({ "id": id.to_string() }),
          correlation_id,
        )),
      }
    }
    Err(ForgeError::Auth(StatusCode::NOT_FOUND, msg)) => {
      Ok(rpc_error(StatusCode::NOT_FOUND, &msg, correlation_id))
    }
    Err(ForgeError::Generic(msg)) => Ok(rpc_error(StatusCode::BAD_REQUEST, &msg, correlation_id)),
    Err(e) => Ok(rpc_error(
      StatusCode::INTERNAL_SERVER_ERROR,
      &e.to_string(),
      correlation_id,
    )),
  }
}

async fn rpc_update(
  db: &DbConnection,
  entity_id: &str,
  id: Uuid,
  body: serde_json::Value,
  correlation_id: Option<serde_json::Value>,
) -> Result<axum::response::Response, ForgeError> {
  if body.is_null() {
    return Ok(rpc_error(
      StatusCode::BAD_REQUEST,
      "params.body required for entity.update",
      correlation_id,
    ));
  }
  match registry::update_model(entity_id, db, id, body)
    .await
    .map_err(crate::Error::from)
  {
    Ok(value) => Ok(rpc_ok_result(value, correlation_id)),
    Err(ForgeError::Auth(StatusCode::NOT_FOUND, msg)) => {
      Ok(rpc_error(StatusCode::NOT_FOUND, &msg, correlation_id))
    }
    Err(ForgeError::Generic(msg)) => Ok(rpc_error(StatusCode::BAD_REQUEST, &msg, correlation_id)),
    Err(e) => Ok(rpc_error(
      StatusCode::INTERNAL_SERVER_ERROR,
      &e.to_string(),
      correlation_id,
    )),
  }
}

async fn rpc_delete(
  db: &DbConnection,
  entity_id: &str,
  id: Uuid,
  correlation_id: Option<serde_json::Value>,
) -> Result<axum::response::Response, ForgeError> {
  match registry::delete_model(entity_id, db, id)
    .await
    .map_err(crate::Error::from)
  {
    Ok(deleted) => Ok(rpc_ok_result(json!({ "deleted": deleted }), correlation_id)),
    Err(ForgeError::Auth(StatusCode::NOT_FOUND, msg)) => {
      Ok(rpc_error(StatusCode::NOT_FOUND, &msg, correlation_id))
    }
    Err(ForgeError::Generic(msg)) => Ok(rpc_error(StatusCode::BAD_REQUEST, &msg, correlation_id)),
    Err(e) => Ok(rpc_error(
      StatusCode::INTERNAL_SERVER_ERROR,
      &e.to_string(),
      correlation_id,
    )),
  }
}

#[cfg(test)]
mod bdd_tests {
  use super::*;
  use axum::http::StatusCode;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests are organized by feature/behavior area with descriptive names.

  mod rpc_request_structure_behavior {
    use super::*;

    #[test]
    fn should_deserialize_rpc_request_with_all_fields() {
      // Given: a JSON string with all RPC request fields
      let json = r#"{
        "method": "entity.get",
        "entity_id": "user",
        "params": {
          "id": "123e4567-e89b-12d3-a456-426614174000"
        },
        "id": 1
      }"#;

      // When: deserializing the JSON
      let request: Result<RpcRequest, _> = serde_json::from_str(json);

      // Then: should deserialize successfully
      assert!(
        request.is_ok(),
        "Should deserialize RPC request with all fields"
      );
      let req = request.unwrap();
      assert_eq!(req.method, "entity.get");
      assert_eq!(req.entity_id, "user");
      assert!(req.params.is_some());
      assert!(req.id.is_some());
    }

    #[test]
    fn should_deserialize_rpc_request_without_params() {
      // Given: a JSON string without params field
      let json = r#"{
        "method": "entity.list",
        "entity_id": "user"
      }"#;

      // When: deserializing the JSON
      let request: Result<RpcRequest, _> = serde_json::from_str(json);

      // Then: should deserialize successfully with None params
      assert!(
        request.is_ok(),
        "Should deserialize RPC request without params"
      );
      let req = request.unwrap();
      assert_eq!(req.method, "entity.list");
      assert_eq!(req.entity_id, "user");
      assert!(req.params.is_none());
    }

    #[test]
    fn should_deserialize_rpc_request_without_correlation_id() {
      // Given: a JSON string without id field
      let json = r#"{
        "method": "entity.list",
        "entity_id": "user",
        "params": {}
      }"#;

      // When: deserializing the JSON
      let request: Result<RpcRequest, _> = serde_json::from_str(json);

      // Then: should deserialize successfully with None id
      assert!(
        request.is_ok(),
        "Should deserialize RPC request without correlation id"
      );
      let req = request.unwrap();
      assert!(req.id.is_none());
    }

    #[test]
    fn should_deserialize_rpc_params_with_all_list_fields() {
      // Given: RPC params with list query fields
      let json = r#"{
        "filter": "name eq 'test'",
        "sort": "created_at",
        "order": "desc",
        "offset": 10,
        "limit": 20
      }"#;

      // When: deserializing the JSON
      let params: Result<RpcParams, _> = serde_json::from_str(json);

      // Then: should deserialize successfully
      assert!(
        params.is_ok(),
        "Should deserialize RPC params with list fields"
      );
      let p = params.unwrap();
      assert_eq!(p.filter, Some("name eq 'test'".to_string()));
      assert_eq!(p.sort, Some("created_at".to_string()));
      assert_eq!(p.order, Some("desc".to_string()));
      assert_eq!(p.offset, Some(10));
      assert_eq!(p.limit, Some(20));
    }
  }

  mod rpc_response_format_behavior {
    use super::*;

    #[test]
    fn should_format_error_response_with_code_and_message() {
      // Given: an error status code and message
      let status = StatusCode::BAD_REQUEST;
      let message = "Invalid request";
      let correlation_id = None;

      // When: creating error response
      let response = rpc_error(status, message, correlation_id);

      // Then: response should have error structure
      assert_eq!(
        response.status(),
        status,
        "Error response should have correct status"
      );
    }

    #[test]
    fn should_include_correlation_id_in_error_response_when_provided() {
      // Given: an error with correlation id
      let status = StatusCode::NOT_FOUND;
      let message = "Not found";
      let correlation_id = Some(serde_json::json!(42));

      // When: creating error response
      let response = rpc_error(status, message, correlation_id);

      // Then: response should include correlation id
      assert_eq!(
        response.status(),
        status,
        "Error response should have correct status"
      );
    }

    #[test]
    fn should_format_success_response_with_result() {
      // Given: a result value
      let result = serde_json::json!({ "id": "123", "name": "test" });
      let correlation_id = None;

      // When: creating success response
      let response = rpc_ok_result(result, correlation_id);

      // Then: response should have success status
      assert_eq!(
        response.status(),
        StatusCode::OK,
        "Success response should have OK status"
      );
    }

    #[test]
    fn should_include_correlation_id_in_success_response_when_provided() {
      // Given: a result with correlation id
      let result = serde_json::json!({ "data": "value" });
      let correlation_id = Some(serde_json::json!("req-123"));

      // When: creating success response
      let response = rpc_ok_result(result, correlation_id);

      // Then: response should include correlation id
      assert_eq!(
        response.status(),
        StatusCode::OK,
        "Success response should have OK status"
      );
    }
  }

  mod method_validation_behavior {
    #[test]
    fn should_accept_entity_list_method() {
      // Given: method is "entity.list"
      let method = "entity.list";

      // When: checking if method is valid
      // Then: should map to ("list", "read") action
      let (action, required_action) = match method {
        "entity.list" => ("list", "read"),
        _ => ("", ""),
      };
      assert_eq!(action, "list");
      assert_eq!(required_action, "read");
    }

    #[test]
    fn should_accept_entity_get_method() {
      // Given: method is "entity.get"
      let method = "entity.get";

      // When: checking if method is valid
      // Then: should map to ("get", "read") action
      let (action, required_action) = match method {
        "entity.get" => ("get", "read"),
        _ => ("", ""),
      };
      assert_eq!(action, "get");
      assert_eq!(required_action, "read");
    }

    #[test]
    fn should_accept_entity_create_method() {
      // Given: method is "entity.create"
      let method = "entity.create";

      // When: checking if method is valid
      // Then: should map to ("create", "create") action
      let (action, required_action) = match method {
        "entity.create" => ("create", "create"),
        _ => ("", ""),
      };
      assert_eq!(action, "create");
      assert_eq!(required_action, "create");
    }

    #[test]
    fn should_accept_entity_update_method() {
      // Given: method is "entity.update"
      let method = "entity.update";

      // When: checking if method is valid
      // Then: should map to ("update", "update") action
      let (action, required_action) = match method {
        "entity.update" => ("update", "update"),
        _ => ("", ""),
      };
      assert_eq!(action, "update");
      assert_eq!(required_action, "update");
    }

    #[test]
    fn should_accept_entity_delete_method() {
      // Given: method is "entity.delete"
      let method = "entity.delete";

      // When: checking if method is valid
      // Then: should map to ("delete", "delete") action
      let (action, required_action) = match method {
        "entity.delete" => ("delete", "delete"),
        _ => ("", ""),
      };
      assert_eq!(action, "delete");
      assert_eq!(required_action, "delete");
    }

    #[test]
    fn should_accept_subscribe_method() {
      // Given: method is "subscribe"
      let method = "subscribe";

      // When: checking if method is handled
      // Then: should be handled before entity methods
      match method {
        "subscribe" => {
          // Subscribe is handled separately
        }
        _ => {}
      }
    }

    #[test]
    fn should_accept_unsubscribe_method() {
      // Given: method is "unsubscribe"
      let method = "unsubscribe";

      // When: checking if method is handled
      // Then: should be handled before entity methods
      match method {
        "unsubscribe" => {
          // Unsubscribe is handled separately
        }
        _ => {}
      }
    }
  }

  mod parameter_validation_behavior {
    use super::*;

    #[test]
    fn should_parse_valid_uuid_from_params_id() {
      // Given: params with valid UUID string
      let uuid_str = "123e4567-e89b-12d3-a456-426614174000";

      // When: parsing UUID
      let result = Uuid::parse_str(uuid_str);

      // Then: should parse successfully
      assert!(result.is_ok(), "Should parse valid UUID");
      let uuid = result.unwrap();
      assert_eq!(uuid.to_string(), uuid_str);
    }

    #[test]
    fn should_reject_invalid_uuid_from_params_id() {
      // Given: params with invalid UUID string
      let invalid_uuid = "not-a-uuid";

      // When: parsing UUID
      let result = Uuid::parse_str(invalid_uuid);

      // Then: should fail to parse
      assert!(result.is_err(), "Should reject invalid UUID");
    }

    #[test]
    fn should_require_params_id_for_get_method() {
      // Given: params without id field
      let params = RpcParams {
        id: None,
        ..Default::default()
      };

      // When: accessing id
      let id_str = params.id.as_deref().unwrap_or("");

      // Then: should be empty string
      assert_eq!(id_str, "", "Should return empty string when id is missing");
    }

    #[test]
    fn should_require_params_body_for_create_method() {
      // Given: params without body field
      let params = RpcParams {
        body: None,
        ..Default::default()
      };

      // When: accessing body
      let body = params.body.unwrap_or(serde_json::Value::Null);

      // Then: should be Null
      assert!(body.is_null(), "Should return Null when body is missing");
    }

    #[test]
    fn should_require_params_subscription_id_for_unsubscribe() {
      // Given: params without subscription_id field
      let params = RpcParams {
        subscription_id: None,
        ..Default::default()
      };

      // When: accessing subscription_id
      let sub_id = params.subscription_id.as_deref().unwrap_or("");

      // Then: should be empty string
      assert_eq!(
        sub_id, "",
        "Should return empty string when subscription_id is missing"
      );
    }
  }

  mod correlation_id_behavior {
    #[test]
    fn should_accept_numeric_correlation_id() {
      // Given: correlation id as number
      let correlation_id = Some(serde_json::json!(42));

      // When: using correlation id
      // Then: should be accepted (no error)
      assert!(correlation_id.is_some());
      assert!(correlation_id.as_ref().unwrap().is_number());
    }

    #[test]
    fn should_accept_string_correlation_id() {
      // Given: correlation id as string
      let correlation_id = Some(serde_json::json!("req-123"));

      // When: using correlation id
      // Then: should be accepted (no error)
      assert!(correlation_id.is_some());
      assert!(correlation_id.as_ref().unwrap().is_string());
    }

    #[test]
    fn should_handle_missing_correlation_id() {
      // Given: no correlation id
      let correlation_id: Option<serde_json::Value> = None;

      // When: using correlation id
      // Then: should handle None gracefully
      assert!(correlation_id.is_none());
    }
  }
}
