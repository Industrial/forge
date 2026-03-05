//! RPC endpoint per docs/technical-choices/07-rpc-parity-with-rest.md.
//! Same entity operations (list, get, create, update, delete) over POST /api/rpc; auth and scope from headers (parity with REST).

use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use db::auth::Backend;
use forge_auth::token_auth::RequireAuth;
use forge_db::DbConnection;
use sea_orm::EntityTrait;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::entity_registry;
use crate::handlers::auth::ScopeFromHeaders;
use crate::handlers::dashboard::require_entity_permission;
use crate::handlers::generic_entity::{list_entity_with_spec, parse_list_query_spec, ListQueryParams};
use crate::subscriptions::{SubscriptionMeta, SubscriptionStore};
use crate::Error as ForgeError;
use db::models::{organization, user};

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

  // Subscribe/unsubscribe (Epic 8): same transport as entity RPC; server-assigned subscription id.
  match method {
    "subscribe" => {
      return rpc_subscribe(user, &db, &scope, &subscriptions, body, correlation_id).await;
    }
    "unsubscribe" => return rpc_unsubscribe(&subscriptions, body, correlation_id).await,
    _ => {}
  }

  if entity_registry::get_entity(entity_id).is_none() {
    return Ok(rpc_error(StatusCode::NOT_FOUND, "Unknown entity", correlation_id));
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
    "create" | "update" | "delete" => Ok(rpc_error(
      StatusCode::NOT_IMPLEMENTED,
      "entity.create / entity.update / entity.delete not yet implemented over RPC",
      correlation_id,
    )),
    _ => Ok(rpc_error(StatusCode::BAD_REQUEST, "invalid method", correlation_id)),
  }
}

/// Subscribe RPC (Epic 8): same query spec as list; server-assigned subscription id; auth and scope per request.
async fn rpc_subscribe(
  user: &user::Model,
  db: &DbConnection,
  scope: &forge_auth::RequestScope,
  store: &SubscriptionStore,
  body: RpcRequest,
  correlation_id: Option<serde_json::Value>,
) -> Result<axum::response::Response, ForgeError> {
  let entity_id = body.entity_id.clone();
  if entity_registry::get_entity(&entity_id).is_none() {
    return Ok(rpc_error(StatusCode::NOT_FOUND, "Unknown entity", correlation_id));
  }
  if let Some(resp) =
    require_entity_permission(user, db, Some(scope), &entity_id, "read").await
  {
    return Ok(resp);
  }
  let meta = SubscriptionMeta {
    entity_id,
    organization_id: scope.organization_id,
    role_id: scope.role_id,
    params: body.params.as_ref().map(|p| {
      serde_json::json!({ "id": p.id, "body": p.body, "subscription_id": p.subscription_id })
    }),
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

fn rpc_ok_result(result: serde_json::Value, id: Option<serde_json::Value>) -> axum::response::Response {
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
      return Ok(rpc_error(
        StatusCode::BAD_REQUEST,
        &msg,
        correlation_id,
      ));
    }
  };
  match list_entity_with_spec(db, entity_id, &spec).await {
    Ok(result) => Ok(rpc_ok_result(result, correlation_id)),
    Err(ForgeError::Auth(status, msg)) => {
      Ok(rpc_error(status, &msg, correlation_id))
    }
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
  match entity_id {
    "organization" => {
      let row = organization::Entity::find_by_id(resource_id)
        .one(db)
        .await
        .map_err(|e| ForgeError::Generic(e.to_string()))?;
      match row {
        Some(r) => Ok(rpc_ok_result(
          json!({
            "id": r.id.to_string(),
            "name": r.name,
            "slug": r.slug,
            "created_at": r.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
            "updated_at": r.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
          }),
          correlation_id,
        )),
        None => Ok(rpc_error(
          StatusCode::NOT_FOUND,
          "Resource not found",
          correlation_id,
        )),
      }
    }
    _ => Ok(rpc_error(
      StatusCode::NOT_IMPLEMENTED,
      "get not implemented for this entity",
      correlation_id,
    )),
  }
}
