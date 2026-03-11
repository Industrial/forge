//! GET/POST /api/auth/users, PATCH/DELETE /api/auth/users/{id}.

use axum::{
  Json,
  extract::{Extension, Path, Query},
  http::StatusCode,
  response::IntoResponse,
};
use chrono::Utc;
use forge_auth::token_auth::{RequireAuth, hash_password};
use forge_live::{Channel, InMemoryLiveBackend, LiveEvent, broadcast_to_channel};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
  Set,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use db::auth::Backend;
use db::models::{membership, org_role, organization, user, user_org_role};

use crate::Error as ForgeError;
use crate::scoped_query::user_find_scoped;

use super::shared::{
  DbFromScope, PERMISSION_USERS_READ, PERMISSION_USERS_WRITE, ScopeFromHeaders, has_global_scope,
  require_permission,
};

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
  pub org_id: Option<Uuid>,
  #[serde(default)]
  pub offset: u64,
  #[serde(default = "default_limit")]
  pub limit: u64,
}

fn default_limit() -> u64 {
  50
}

/// GET /api/auth/users — list users. Optional ?org_id= for scope. Requires user.read.
pub async fn list_users(
  ScopeFromHeaders(scope, _): ScopeFromHeaders<user::Model>,
  auth: RequireAuth<Backend, user::Model>,
  DbFromScope(db): DbFromScope,
  Query(q): Query<ListUsersQuery>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_USERS_READ, Some(&scope)).await {
    return Ok(resp);
  }
  let scope_opt: Option<&forge_auth::RequestScope> =
    if has_global_scope(&db, user, PERMISSION_USERS_READ).await {
      None
    } else {
      Some(&scope)
    };
  let select = user_find_scoped(&db, scope_opt)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  let total = select
    .clone()
    .count(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  let limit = q.limit.min(200);
  let users = select
    .order_by_asc(user::Column::Email)
    .offset(q.offset)
    .limit(limit)
    .all(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  let user_ids: Vec<Uuid> = users.iter().map(|u| u.id).collect();
  let (_org_map, _role_map, memberships_by_user) = if user_ids.is_empty() {
    (
      std::collections::HashMap::new(),
      std::collections::HashMap::new(),
      std::collections::HashMap::<Uuid, Vec<(Uuid, String, Vec<String>)>>::new(),
    )
  } else {
    let uors = user_org_role::Entity::find()
      .filter(user_org_role::Column::UserId.is_in(user_ids.clone()))
      .all(&db)
      .await
      .map_err(|e| ForgeError::Generic(e.to_string()))?;
    let org_ids: Vec<Uuid> = uors
      .iter()
      .map(|u| u.org_id)
      .collect::<std::collections::HashSet<_>>()
      .into_iter()
      .collect();
    let role_ids: Vec<Uuid> = uors
      .iter()
      .map(|u| u.role_id)
      .collect::<std::collections::HashSet<_>>()
      .into_iter()
      .collect();
    let orgs = if org_ids.is_empty() {
      vec![]
    } else {
      organization::Entity::find()
        .filter(organization::Column::Id.is_in(org_ids))
        .all(&db)
        .await
        .map_err(|e| ForgeError::Generic(e.to_string()))?
    };
    let roles = if role_ids.is_empty() {
      vec![]
    } else {
      org_role::Entity::find()
        .filter(org_role::Column::Id.is_in(role_ids))
        .all(&db)
        .await
        .map_err(|e| ForgeError::Generic(e.to_string()))?
    };
    let org_map: std::collections::HashMap<Uuid, String> =
      orgs.into_iter().map(|o| (o.id, o.name)).collect();
    let role_map: std::collections::HashMap<Uuid, String> =
      roles.into_iter().map(|r| (r.id, r.name.clone())).collect();
    let mut memberships_by_user: std::collections::HashMap<Uuid, Vec<(Uuid, String, Vec<String>)>> =
      std::collections::HashMap::new();
    for uor in &uors {
      let org_name = org_map
        .get(&uor.org_id)
        .cloned()
        .unwrap_or_else(|| uor.org_id.to_string());
      let role_name = role_map
        .get(&uor.role_id)
        .cloned()
        .unwrap_or_else(|| uor.role_id.to_string());
      memberships_by_user.entry(uor.user_id).or_default().push((
        uor.org_id,
        org_name,
        vec![role_name],
      ));
    }
    for (_, per_org) in memberships_by_user.iter_mut() {
      per_org.sort_by_key(|(org_id, _, _)| *org_id);
      let mut merged: Vec<(Uuid, String, Vec<String>)> = Vec::new();
      for (org_id, org_name, roles) in per_org.drain(..) {
        if let Some(last) = merged.last_mut()
          && last.0 == org_id
        {
          if !last.2.contains(&roles[0]) {
            last.2.push(roles[0].clone());
          }
          continue;
        }
        merged.push((org_id, org_name, roles));
      }
      *per_org = merged;
    }
    (org_map, role_map, memberships_by_user)
  };
  let list: Vec<serde_json::Value> = users
    .into_iter()
    .map(|u| {
      let memberships: Vec<serde_json::Value> = memberships_by_user
        .get(&u.id)
        .map(|per_org| {
          per_org
            .iter()
            .map(|(org_id, org_name, role_names)| {
              serde_json::json!({
                "org_id": org_id.to_string(),
                "org_name": org_name,
                "roles": role_names,
              })
            })
            .collect()
        })
        .unwrap_or_default();
      serde_json::json!({
        "id": u.id.to_string(),
        "email": u.email,
        "is_active": u.is_active,
        "is_admin": u.is_admin,
        "created_at": u.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
        "memberships": memberships,
      })
    })
    .collect();
  Ok(Json(serde_json::json!({ "users": list, "total": total })).into_response())
}

#[derive(Deserialize)]
pub struct CreateUserBody {
  pub email: String,
  pub password: String,
  pub org_id: Uuid,
  pub role_ids: Vec<Uuid>,
}

/// POST /api/auth/users — create user with org membership and roles. Requires user.create.
pub async fn create_user(
  ScopeFromHeaders(scope, _): ScopeFromHeaders<user::Model>,
  auth: RequireAuth<Backend, user::Model>,
  DbFromScope(db): DbFromScope,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<CreateUserBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_USERS_WRITE, Some(&scope)).await {
    return Ok(resp);
  }
  let email = payload.email.trim();
  if email.is_empty() || !email.contains('@') {
    return Ok(
      (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": "Valid email is required" })),
      )
        .into_response(),
    );
  }
  if payload.password.len() < 8 {
    return Ok(
      (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": "Password must be at least 8 characters" })),
      )
        .into_response(),
    );
  }
  let org_id = if has_global_scope(&db, user, PERMISSION_USERS_WRITE).await {
    payload.org_id
  } else {
    if scope.organization_id != payload.org_id {
      return Ok(
        (
          StatusCode::FORBIDDEN,
          Json(serde_json::json!({ "error": "Can only add users to your current organization" })),
        )
          .into_response(),
      );
    }
    payload.org_id
  };
  if payload.role_ids.is_empty() {
    return Ok(
      (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": "At least one role is required" })),
      )
        .into_response(),
    );
  }
  for role_id in &payload.role_ids {
    let role_row = org_role::Entity::find_by_id(*role_id)
      .one(&db)
      .await
      .map_err(|e| ForgeError::Generic(e.to_string()))?;
    match role_row {
      Some(r) if r.org_id == org_id => {}
      _ => {
        return Ok(
          (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "Each role_id must belong to the selected organization" })),
          )
            .into_response(),
        );
      }
    }
  }
  if user::Entity::find()
    .filter(user::Column::Email.eq(email))
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?
    .is_some()
  {
    return Ok(
      (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(serde_json::json!({ "error": "Email already in use" })),
      )
        .into_response(),
    );
  }
  let password_hash =
    hash_password(&payload.password).map_err(|e| ForgeError::Generic(e.to_string()))?;
  let now = Utc::now().naive_utc();
  let user_id = Uuid::new_v4();
  let membership_id = Uuid::new_v4();
  let user_model = user::ActiveModel {
    id: Set(user_id),
    email: Set(email.to_string()),
    password_hash: Set(password_hash),
    is_active: Set(true),
    is_admin: Set(false),
    current_org_id: Set(Some(org_id)),
    current_role: Set(Some(
      org_role::Entity::find_by_id(payload.role_ids[0])
        .one(&db)
        .await
        .ok()
        .flatten()
        .map(|r| r.name)
        .unwrap_or_else(|| "viewer".to_string()),
    )),
    created_at: Set(now),
    updated_at: Set(now),
  };
  user::Entity::insert(user_model)
    .exec(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  let mem_model = membership::ActiveModel {
    id: Set(membership_id),
    user_id: Set(user_id),
    org_id: Set(org_id),
    created_at: Set(now),
    updated_at: Set(now),
  };
  membership::Entity::insert(mem_model)
    .exec(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  for role_id in &payload.role_ids {
    let uor_id = Uuid::new_v4();
    let uor = user_org_role::ActiveModel {
      id: Set(uor_id),
      user_id: Set(user_id),
      org_id: Set(org_id),
      role_id: Set(*role_id),
      created_at: Set(now),
      updated_at: Set(now),
    };
    user_org_role::Entity::insert(uor)
      .exec(&db)
      .await
      .map_err(|e| ForgeError::Generic(e.to_string()))?;
  }
  let role_names: Vec<String> = org_role::Entity::find()
    .filter(org_role::Column::Id.is_in(payload.role_ids.clone()))
    .all(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?
    .into_iter()
    .map(|r| r.name)
    .collect();
  if let Some(ref backend) = live_backend {
    let _ = broadcast_to_channel(
      backend,
      &Channel::org_resource(org_id, "users"),
      &LiveEvent::UsersUpdated {
        user_id: Some(user_id),
        org_id: Some(org_id),
      },
    )
    .await;
  }
  Ok(
    (
      StatusCode::CREATED,
      Json(serde_json::json!({
        "id": user_id.to_string(),
        "email": email,
        "is_active": true,
        "is_admin": false,
        "created_at": now.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
        "memberships": [{ "org_id": org_id.to_string(), "roles": role_names }],
      })),
    )
      .into_response(),
  )
}

#[derive(Deserialize)]
pub struct UpdateUserBody {
  pub email: Option<String>,
  pub is_active: Option<bool>,
}

/// PATCH /api/auth/users/{id} — update user. Requires user.update.
pub async fn update_user(
  ScopeFromHeaders(scope, _): ScopeFromHeaders<user::Model>,
  auth: RequireAuth<Backend, user::Model>,
  DbFromScope(db): DbFromScope,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Path(id): Path<Uuid>,
  Json(payload): Json<UpdateUserBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_USERS_WRITE, Some(&scope)).await {
    return Ok(resp);
  }
  let u = user::Entity::find_by_id(id)
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?
    .ok_or_else(|| ForgeError::Generic("User not found".into()))?;
  let mut am: user::ActiveModel = u.into();
  if let Some(email) = payload.email {
    let t = email.trim();
    if !t.is_empty() && t.contains('@') {
      am.email = Set(t.to_string());
    }
  }
  if let Some(is_active) = payload.is_active {
    am.is_active = Set(is_active);
  }
  am.updated_at = Set(Utc::now().naive_utc());
  am.update(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if let Some(ref backend) = live_backend {
    let _ = broadcast_to_channel(
      backend,
      &Channel::org_resource(scope.organization_id, "users"),
      &LiveEvent::UsersUpdated {
        user_id: Some(id),
        org_id: Some(scope.organization_id),
      },
    )
    .await;
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

/// DELETE /api/auth/users/{id} — delete user. Requires user.delete.
pub async fn delete_user(
  ScopeFromHeaders(scope, _): ScopeFromHeaders<user::Model>,
  auth: RequireAuth<Backend, user::Model>,
  DbFromScope(db): DbFromScope,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_USERS_WRITE, Some(&scope)).await {
    return Ok(resp);
  }
  membership::Entity::delete_many()
    .filter(membership::Column::UserId.eq(id))
    .exec(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  let result = user::Entity::delete_by_id(id)
    .exec(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if result.rows_affected == 0 {
    return Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "User not found" })),
      )
        .into_response(),
    );
  }
  if let Some(ref backend) = live_backend {
    let _ = broadcast_to_channel(
      backend,
      &Channel::org_resource(scope.organization_id, "users"),
      &LiveEvent::UsersUpdated {
        user_id: Some(id),
        org_id: Some(scope.organization_id),
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
  fn list_users_query_default_limit() {
    let q = ListUsersQuery {
      org_id: None,
      offset: 0,
      limit: 50,
    };
    assert_eq!(q.limit, 50);
  }

  #[test]
  fn create_user_body_requires_fields() {
    let _ = CreateUserBody {
      email: "a@b.com".to_string(),
      password: "password123".to_string(),
      org_id: Uuid::nil(),
      role_ids: vec![Uuid::nil()],
    };
  }
}
