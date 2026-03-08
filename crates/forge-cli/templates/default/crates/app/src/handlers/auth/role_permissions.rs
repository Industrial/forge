//! GET/POST/DELETE /api/auth/role-permissions. Flat: scope, org_id, role_name in query/body.

use axum::{
  Json,
  extract::{Extension, Query, State},
  http::StatusCode,
  response::IntoResponse,
};
use forge_auth::token_auth::RequireAuth;
use forge_db::DbConnection;
use forge_live::{Channel, InMemoryLiveBackend, LiveEvent, broadcast_to_channel};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use db::auth::Backend;
use db::models::{role_permission, user};

use crate::Error as ForgeError;
use crate::permissions::dashboard_permissions;

use super::shared::{
  PERMISSION_READ, PERMISSION_WRITE, ScopeFromHeaders, has_global_scope, require_permission,
};

#[derive(Debug, Deserialize)]
pub struct ListRolePermissionsQuery {
  pub scope: Option<String>,
  pub org_id: Option<Uuid>,
  pub role_name: Option<String>,
}

/// GET /api/auth/role-permissions — list assignments. ?scope=global or ?org_id=&role_name=. Requires permission.read.
pub async fn list_role_permissions(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Query(q): Query<ListRolePermissionsQuery>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_READ, Some(&scope)).await {
    return Ok(resp);
  }
  let rows = if has_global_scope(&db, user, PERMISSION_READ).await {
    let mut query = role_permission::Entity::find();
    if let Some(ref scope_str) = q.scope {
      query = query.filter(role_permission::Column::Scope.eq(scope_str.as_str()));
    }
    if let Some(org_id) = q.org_id {
      query = query.filter(role_permission::Column::OrgId.eq(org_id));
    }
    if let Some(ref role_name) = q.role_name {
      query = query.filter(role_permission::Column::RoleName.eq(role_name.as_str()));
    }
    query.all(&db).await
  } else {
    role_permission::Entity::find()
      .filter(role_permission::Column::Scope.eq("org"))
      .filter(role_permission::Column::OrgId.eq(scope.organization_id))
      .all(&db)
      .await
  };
  let rows = rows.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let list: Vec<serde_json::Value> = rows
    .into_iter()
    .map(|r| {
      serde_json::json!({
        "id": r.id.to_string(),
        "scope": r.scope,
        "role_name": r.role_name,
        "permission_key": r.permission_key,
        "org_id": r.org_id,
      })
    })
    .collect();
  Ok(Json(serde_json::json!({ "assignments": list })).into_response())
}

#[derive(Deserialize)]
pub struct AddRolePermissionBody {
  pub scope: String,
  pub role_name: String,
  pub permission_key: String,
  pub org_id: Option<Uuid>,
}

/// POST /api/auth/role-permissions — add assignment. Requires permission.write.
pub async fn add_role_permission(
  ScopeFromHeaders(req_scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<AddRolePermissionBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_WRITE, Some(&req_scope)).await {
    return Ok(resp);
  }
  let scope = payload.scope.trim();
  let role_name = payload.role_name.trim();
  let permission_key = payload.permission_key.trim();
  if scope != "org" && scope != "global" {
    return Ok(
      (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": "scope must be 'org' or 'global'" })),
      )
        .into_response(),
    );
  }
  if !dashboard_permissions().contains(&permission_key) {
    return Ok(
      (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({
          "error": "invalid permission_key",
          "message": "Permission key is not in the allowed set"
        })),
      )
        .into_response(),
    );
  }
  let global_perm = has_global_scope(&db, user, PERMISSION_WRITE).await;
  let org_id_opt = if global_perm {
    if scope == "org" {
      payload.org_id.or(Some(req_scope.organization_id))
    } else {
      None
    }
  } else {
    if scope != "org" {
      return Ok(
        (
          StatusCode::FORBIDDEN,
          Json(serde_json::json!({ "error": "only org scope allowed" })),
        )
          .into_response(),
      );
    }
    Some(req_scope.organization_id)
  };
  let id = Uuid::new_v4();
  let mut model = role_permission::ActiveModel {
    id: Set(id),
    scope: Set(scope.to_string()),
    role_name: Set(role_name.to_string()),
    permission_key: Set(permission_key.to_string()),
    ..Default::default()
  };
  model.org_id = Set(org_id_opt);
  if let Err(e) = role_permission::Entity::insert(model).exec(&db).await {
    if matches!(
      e.sql_err(),
      Some(sea_orm::SqlErr::UniqueConstraintViolation(_))
    ) {
      return Ok(
        (
          StatusCode::UNPROCESSABLE_ENTITY,
          Json(serde_json::json!({ "error": "assignment already exists" })),
        )
          .into_response(),
      );
    }
    return Err(ForgeError::Generic(e.to_string()));
  }
  if let (Some(backend), Some(org_id)) = (live_backend.as_ref(), org_id_opt) {
    let _ = broadcast_to_channel(
      backend,
      &Channel::org_resource(org_id, "role_permissions"),
      &LiveEvent::ResourceChanged {
        resource: "role_permissions".into(),
        id,
        action: Some("created".into()),
      },
    )
    .await;
  }
  Ok((StatusCode::CREATED, Json(serde_json::json!({ "ok": true }))).into_response())
}

#[derive(Deserialize)]
pub struct DeleteRolePermissionBody {
  pub scope: String,
  pub role_name: String,
  pub permission_key: String,
  pub org_id: Option<Uuid>,
}

/// DELETE /api/auth/role-permissions — remove assignment (body). Requires permission.write.
pub async fn delete_role_permission(
  ScopeFromHeaders(req_scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<DeleteRolePermissionBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_WRITE, Some(&req_scope)).await {
    return Ok(resp);
  }
  let scope = payload.scope.trim();
  let role_name = payload.role_name.trim();
  let permission_key = payload.permission_key.trim();
  let global_perm = has_global_scope(&db, user, PERMISSION_WRITE).await;
  let org_id_opt = if global_perm {
    if scope == "org" {
      if payload.org_id.is_none() {
        return Ok(
          (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "org_id required when scope=org" })),
          )
            .into_response(),
        );
      }
      payload.org_id
    } else {
      None
    }
  } else {
    if scope != "org" {
      return Ok(
        (
          StatusCode::FORBIDDEN,
          Json(serde_json::json!({ "error": "only org scope allowed" })),
        )
          .into_response(),
      );
    }
    Some(req_scope.organization_id)
  };
  let mut q = role_permission::Entity::delete_many()
    .filter(role_permission::Column::Scope.eq(scope))
    .filter(role_permission::Column::RoleName.eq(role_name))
    .filter(role_permission::Column::PermissionKey.eq(permission_key));
  match org_id_opt {
    Some(id) => q = q.filter(role_permission::Column::OrgId.eq(id)),
    None => q = q.filter(role_permission::Column::OrgId.is_null()),
  }
  let result = q
    .exec(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if result.rows_affected == 0 {
    return Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "assignment not found" })),
      )
        .into_response(),
    );
  }
  if let (Some(backend), Some(org_id)) = (live_backend.as_ref(), org_id_opt) {
    let _ = broadcast_to_channel(
      backend,
      &Channel::org_resource(org_id, "role_permissions"),
      &LiveEvent::ResourceChanged {
        resource: "role_permissions".into(),
        id: Uuid::nil(),
        action: Some("deleted".into()),
      },
    )
    .await;
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn list_role_permissions_handler_exists() {
    let _ = list_role_permissions;
  }
}
