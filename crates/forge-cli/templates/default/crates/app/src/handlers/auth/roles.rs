//! GET/POST /api/auth/roles, PATCH/DELETE /api/auth/roles/{id}. Flat: org_id in query/body.

use axum::{
  Json,
  extract::{Extension, Path, Query, State},
  http::StatusCode,
  response::IntoResponse,
};
use forge_auth::token_auth::RequireAuth;
use forge_db::DbConnection;
use forge_live::{Channel, InMemoryLiveBackend, LiveEvent, broadcast_to_channel};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use db::auth::Backend;
use db::models::{org_role, user, user_org_role};

use crate::Error as ForgeError;

use super::shared::{
  PERMISSION_ROLES_READ, PERMISSION_ROLES_WRITE, ScopeFromHeaders, has_global_scope,
  require_permission,
};

#[derive(Debug, Deserialize)]
pub struct ListRolesQuery {
  pub org_id: Option<Uuid>,
}

/// GET /api/auth/roles — list roles. ?org_id= for org-scoped. Requires role.read.
pub async fn list_roles(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Query(q): Query<ListRolesQuery>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_ROLES_READ, Some(&scope)).await {
    return Ok(resp);
  }
  let rows = if has_global_scope(&db, user, PERMISSION_ROLES_READ).await {
    org_role::Entity::find()
      .order_by_asc(org_role::Column::OrgId)
      .order_by_asc(org_role::Column::Name)
      .all(&db)
      .await
  } else {
    let org_id = q.org_id.unwrap_or(scope.organization_id);
    if org_id != scope.organization_id {
      return Ok(
        (
          StatusCode::FORBIDDEN,
          Json(serde_json::json!({ "error": "Forbidden" })),
        )
          .into_response(),
      );
    }
    org_role::Entity::find()
      .filter(org_role::Column::OrgId.eq(org_id))
      .order_by_asc(org_role::Column::Name)
      .all(&db)
      .await
  };
  let rows = rows.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let list: Vec<serde_json::Value> = rows
    .into_iter()
    .map(|r| {
      serde_json::json!({
        "id": r.id.to_string(),
        "org_id": r.org_id.to_string(),
        "name": r.name,
        "display_name": r.display_name,
        "created_at": r.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
        "updated_at": r.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
      })
    })
    .collect();
  Ok(Json(serde_json::json!({ "roles": list })).into_response())
}

#[derive(Deserialize)]
pub struct CreateRoleBody {
  pub org_id: Uuid,
  pub name: String,
  pub display_name: Option<String>,
}

/// POST /api/auth/roles — create role. Requires role.create.
pub async fn create_role(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<CreateRoleBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_ROLES_WRITE, Some(&scope)).await {
    return Ok(resp);
  }
  let org_id = if has_global_scope(&db, user, PERMISSION_ROLES_WRITE).await {
    payload.org_id
  } else {
    if scope.organization_id != payload.org_id {
      return Ok(
        (
          StatusCode::FORBIDDEN,
          Json(
            serde_json::json!({ "error": "Can only create roles in your current organization" }),
          ),
        )
          .into_response(),
      );
    }
    payload.org_id
  };
  let name = payload.name.trim();
  if name.is_empty() {
    return Ok(
      (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": "name is required" })),
      )
        .into_response(),
    );
  }
  let existing = org_role::Entity::find()
    .filter(org_role::Column::OrgId.eq(org_id))
    .filter(org_role::Column::Name.eq(name))
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if existing.is_some() {
    return Ok(
      (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": "A role with this name already exists in this organization" })),
      )
        .into_response(),
    );
  }
  let now = chrono::Utc::now().naive_utc();
  let id = Uuid::new_v4();
  let display_name = payload
    .display_name
    .as_ref()
    .map(|s| s.trim())
    .filter(|s| !s.is_empty())
    .map(|s| s.to_string());
  let model = org_role::ActiveModel {
    id: Set(id),
    org_id: Set(org_id),
    name: Set(name.to_string()),
    display_name: Set(display_name.clone()),
    created_at: Set(now),
    updated_at: Set(now),
  };
  org_role::Entity::insert(model)
    .exec(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if let Some(ref backend) = live_backend {
    let _ = broadcast_to_channel(
      backend,
      &Channel::org_resource(org_id, "roles"),
      &LiveEvent::ResourceChanged {
        resource: "roles".into(),
        id,
        action: Some("created".into()),
      },
    )
    .await;
  }
  Ok(
    (
      StatusCode::CREATED,
      Json(serde_json::json!({
        "id": id.to_string(),
        "org_id": org_id.to_string(),
        "name": name,
        "display_name": display_name,
        "created_at": now.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
        "updated_at": now.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
      })),
    )
      .into_response(),
  )
}

#[derive(Deserialize)]
pub struct UpdateRoleBody {
  pub name: Option<String>,
  pub display_name: Option<String>,
}

/// PATCH /api/auth/roles/{id}. Requires role.update.
pub async fn update_role(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Path(id): Path<Uuid>,
  Json(payload): Json<UpdateRoleBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_ROLES_WRITE, Some(&scope)).await {
    return Ok(resp);
  }
  let role = org_role::Entity::find_by_id(id)
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?
    .ok_or_else(|| ForgeError::Generic("Role not found".into()))?;
  let can_write_org = has_global_scope(&db, user, PERMISSION_ROLES_WRITE).await
    || scope.organization_id == role.org_id;
  if !can_write_org {
    return Ok(
      (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": "Can only update roles in your current organization" })),
      )
        .into_response(),
    );
  }
  let org_id = role.org_id;
  let role_id = role.id;
  let mut am: org_role::ActiveModel = role.into();
  if let Some(name) = payload.name {
    let t = name.trim();
    if !t.is_empty() {
      let existing = org_role::Entity::find()
        .filter(org_role::Column::OrgId.eq(org_id))
        .filter(org_role::Column::Name.eq(t))
        .filter(org_role::Column::Id.ne(role_id))
        .one(&db)
        .await
        .map_err(|e| ForgeError::Generic(e.to_string()))?;
      if existing.is_some() {
        return Ok(
          (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "A role with this name already exists" })),
          )
            .into_response(),
        );
      }
      am.name = Set(t.to_string());
    }
  }
  if let Some(display_name) = payload.display_name {
    am.display_name = Set(Some(display_name.trim().to_string()).filter(|s| !s.is_empty()));
  }
  am.updated_at = Set(chrono::Utc::now().naive_utc());
  am.update(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if let Some(ref backend) = live_backend {
    let _ = broadcast_to_channel(
      backend,
      &Channel::org_resource(org_id, "roles"),
      &LiveEvent::ResourceChanged {
        resource: "roles".into(),
        id: role_id,
        action: Some("updated".into()),
      },
    )
    .await;
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

/// DELETE /api/auth/roles/{id}. Requires role.delete.
pub async fn delete_role(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_ROLES_WRITE, Some(&scope)).await {
    return Ok(resp);
  }
  let role = org_role::Entity::find_by_id(id)
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?
    .ok_or_else(|| ForgeError::Generic("Role not found".into()))?;
  let can_write_org = has_global_scope(&db, user, PERMISSION_ROLES_WRITE).await
    || scope.organization_id == role.org_id;
  if !can_write_org {
    return Ok(
      (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": "Can only delete roles in your current organization" })),
      )
        .into_response(),
    );
  }
  let count = user_org_role::Entity::find()
    .filter(user_org_role::Column::RoleId.eq(id))
    .count(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if count > 0 {
    return Ok(
      (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": "Cannot delete role: one or more users have this role. Remove assignments first." })),
      )
        .into_response(),
    );
  }
  org_role::Entity::delete_by_id(id)
    .exec(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if let Some(ref backend) = live_backend {
    let _ = broadcast_to_channel(
      backend,
      &Channel::org_resource(role.org_id, "roles"),
      &LiveEvent::ResourceChanged {
        resource: "roles".into(),
        id,
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
  fn list_roles_handler_exists() {
    let _ = list_roles;
  }
}
