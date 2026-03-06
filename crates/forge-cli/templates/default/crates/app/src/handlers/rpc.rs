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
use crate::handlers::dashboard::require_entity_permission;
use crate::handlers::generic_entity::{
  ListQueryParams, list_entity_with_spec, parse_list_query_spec,
};
use crate::query_spec::{validate_filter_cond, validate_sort_field};
use crate::registry;
use crate::subscriptions::{SubscriptionMeta, SubscriptionStore};
use db::models::user;

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
  /// For unsubscribe: server-assigned subscription id.
  pub subscription_id: Option<String>,
  /// For entity.list: same as REST query params (filter, sort, order, offset, limit).
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
    "list" => rpc_list(&db, entity_id, body.params.as_ref(), correlation_id).await,
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
  if let Some(ref sort) = spec.sort {
    if let Err(msg) = validate_sort_field(&sort.field, allowed_sort) {
      return Ok(rpc_error(StatusCode::BAD_REQUEST, &msg, correlation_id));
    }
  }
  let params = serde_json::to_value(&spec).ok();
  let meta = SubscriptionMeta {
    entity_id,
    organization_id: scope.organization_id,
    role_id: scope.role_id,
    params,
  };
  let subscription_id = store.subscribe(meta);
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
) -> Result<axum::response::Response, ForgeError> {
  let list_params = ListQueryParams {
    expand: None,
    include: None,
    filter: params.and_then(|p| p.filter.clone()),
    sort: params.and_then(|p| p.sort.clone()),
    order: params.and_then(|p| p.order.clone()),
    offset: params.and_then(|p| p.offset),
    limit: params.and_then(|p| p.limit),
  };
  let spec = match parse_list_query_spec(&list_params) {
    Ok(s) => s,
    Err(msg) => {
      return Ok(rpc_error(StatusCode::BAD_REQUEST, &msg, correlation_id));
    }
  };
  match list_entity_with_spec(db, entity_id, &spec).await {
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
