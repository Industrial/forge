//! REST API: org-scoped routes use [ScopeFromHeaders] (X-Organization-Id, X-Role-Id); others require Bearer where applicable.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use axum::Json;
use chrono::NaiveDateTime;
use forge_auth::token_auth::hash_password;
use forge_db::DbConnection;
use crate::Error as ForgeError;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set};
use serde::Deserialize;
use uuid::Uuid;

use crate::handlers::auth::ScopeFromHeaders;
use crate::permissions::DASHBOARD_PERMISSIONS;
use db::models::{audit_log, membership, org_role, organization, role_permission, user, user_org_role};

// ---- Permissions (code-defined keys) ----
/// GET /api/permissions — list known permission keys (code-defined). Read-only; no auth or scope required.
pub async fn list_permissions(State(_db): State<DbConnection>) -> Result<impl IntoResponse, ForgeError> {
  let list: Vec<&str> = DASHBOARD_PERMISSIONS.to_vec();
  Ok(Json(serde_json::json!({ "permissions": list })).into_response())
}

fn slug_from_name(name: &str) -> String {
  name
    .to_lowercase()
    .chars()
    .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { '-' })
    .collect::<String>()
    .split_whitespace()
    .filter(|s| !s.is_empty())
    .collect::<Vec<_>>()
    .join("-")
}

// ---- Users ----
pub async fn list_users(State(db): State<DbConnection>) -> Result<impl IntoResponse, ForgeError> {
  let users = user::Entity::find().all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let all_uors = user_org_role::Entity::find().all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let role_ids: Vec<Uuid> = all_uors.iter().map(|x| x.role_id).collect::<std::collections::HashSet<_>>().into_iter().collect();
  let roles: Vec<org_role::Model> = if role_ids.is_empty() { vec![] } else {
    org_role::Entity::find().filter(org_role::Column::Id.is_in(role_ids)).all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?
  };
  let org_ids: Vec<Uuid> = all_uors.iter().map(|x| x.org_id).collect::<std::collections::HashSet<_>>().into_iter().collect();
  let orgs: Vec<organization::Model> = if org_ids.is_empty() { vec![] } else {
    organization::Entity::find().filter(organization::Column::Id.is_in(org_ids)).all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?
  };
  let role_map: std::collections::HashMap<Uuid, org_role::Model> = roles.into_iter().map(|r| (r.id, r)).collect();
  let org_map: std::collections::HashMap<Uuid, organization::Model> = orgs.into_iter().map(|o| (o.id, o)).collect();
  let list: Vec<serde_json::Value> = users.into_iter().map(|u| {
    let uors_for_user: Vec<_> = all_uors.iter().filter(|x| x.user_id == u.id).collect();
    let mut org_to_roles: std::collections::HashMap<Uuid, Vec<String>> = std::collections::HashMap::new();
    for x in &uors_for_user {
      if let Some(role) = role_map.get(&x.role_id) {
        org_to_roles.entry(x.org_id).or_default().push(role.name.clone());
      }
    }
    let mems: Vec<serde_json::Value> = org_to_roles.into_iter().map(|(org_id, role_names)| {
      let org_name = org_map.get(&org_id).map(|o| o.name.clone()).unwrap_or_else(|| "—".to_string());
      serde_json::json!({ "org_id": org_id.to_string(), "org_name": org_name, "roles": role_names })
    }).collect();
    serde_json::json!({
      "id": u.id.to_string(),
      "email": u.email,
      "is_active": u.is_active,
      "is_admin": u.is_admin,
      "created_at": u.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
      "memberships": mems,
    })
  }).collect();
  Ok(Json(serde_json::json!({ "users": list })).into_response())
}

#[derive(Deserialize)]
pub struct CreateUserBody {
  pub email: String,
  pub password: String,
  /// Role IDs in the current organization (from scope headers).
  pub role_ids: Vec<Uuid>,
}

pub async fn create_user(ScopeFromHeaders(scope): ScopeFromHeaders, Extension(db): Extension<DbConnection>, Json(payload): Json<CreateUserBody>) -> Result<impl IntoResponse, ForgeError> {
  let org_id = scope.organization_id;
  let email = payload.email.trim();
  if email.is_empty() || !email.contains('@') {
    return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "Valid email is required" }))).into_response());
  }
  if payload.password.len() < 8 {
    return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "Password must be at least 8 characters" }))).into_response());
  }
  if payload.role_ids.is_empty() {
    return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "At least one role is required" }))).into_response());
  }
  for role_id in &payload.role_ids {
    let r = org_role::Entity::find_by_id(*role_id).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
    if r.map(|r| r.org_id != org_id).unwrap_or(true) {
      return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "Each role_id must belong to the organization" }))).into_response());
    }
  }
  if user::Entity::find().filter(user::Column::Email.eq(email)).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?.is_some() {
    return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "Email already in use" }))).into_response());
  }
  let password_hash = hash_password(&payload.password).map_err(|e| ForgeError::Generic(e.to_string()))?;
  let now = chrono::Utc::now().naive_utc();
  let user_id = Uuid::new_v4();
  let role_name = org_role::Entity::find_by_id(payload.role_ids[0]).one(&db).await.ok().flatten().map(|r| r.name).unwrap_or_else(|| "viewer".to_string());
  user::Entity::insert(user::ActiveModel {
    id: Set(user_id),
    email: Set(email.to_string()),
    password_hash: Set(password_hash),
    is_active: Set(true),
    is_admin: Set(false),
    current_org_id: Set(Some(org_id)),
    current_role: Set(Some(role_name)),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  }).exec(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  membership::Entity::insert(membership::ActiveModel {
    id: Set(Uuid::new_v4()),
    user_id: Set(user_id),
    org_id: Set(org_id),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  }).exec(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  for role_id in &payload.role_ids {
    user_org_role::Entity::insert(user_org_role::ActiveModel {
      id: Set(Uuid::new_v4()),
      user_id: Set(user_id),
      org_id: Set(org_id),
      role_id: Set(*role_id),
      created_at: Set(now),
      updated_at: Set(now),
      ..Default::default()
    }).exec(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  }
  let role_names: Vec<String> = org_role::Entity::find().filter(org_role::Column::Id.is_in(payload.role_ids.clone())).all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?.into_iter().map(|r| r.name).collect();
  Ok((StatusCode::CREATED, Json(serde_json::json!({
    "id": user_id.to_string(),
    "email": email,
    "is_active": true,
    "is_admin": false,
    "created_at": now.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
    "memberships": [{ "org_id": org_id.to_string(), "roles": role_names }],
  }))).into_response())
}

pub async fn get_user(Path(id): Path<Uuid>, State(db): State<DbConnection>) -> Result<impl IntoResponse, ForgeError> {
  let u = user::Entity::find_by_id(id).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let Some(u) = u else {
    return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "User not found" }))).into_response());
  };
  let uors = user_org_role::Entity::find().filter(user_org_role::Column::UserId.eq(id)).all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let role_ids: Vec<Uuid> = uors.iter().map(|x| x.role_id).collect();
  let roles = org_role::Entity::find().filter(org_role::Column::Id.is_in(role_ids)).all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let org_ids: Vec<Uuid> = uors.iter().map(|x| x.org_id).collect::<std::collections::HashSet<_>>().into_iter().collect();
  let orgs = organization::Entity::find().filter(organization::Column::Id.is_in(org_ids)).all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let role_map: std::collections::HashMap<Uuid, _> = roles.into_iter().map(|r| (r.id, r)).collect();
  let org_map: std::collections::HashMap<Uuid, _> = orgs.into_iter().map(|o| (o.id, o)).collect();
  let mems: Vec<serde_json::Value> = uors.iter().map(|x| {
    let r = role_map.get(&x.role_id).map(|r| r.name.as_str()).unwrap_or("—");
    let o = org_map.get(&x.org_id).map(|o| o.name.as_str()).unwrap_or("—");
    serde_json::json!({ "org_id": x.org_id.to_string(), "org_name": o, "roles": [r] })
  }).collect();
  Ok(Json(serde_json::json!({
    "id": u.id.to_string(),
    "email": u.email,
    "is_active": u.is_active,
    "is_admin": u.is_admin,
    "created_at": u.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
    "memberships": mems,
  })).into_response())
}

#[derive(Deserialize)]
pub struct UpdateUserBody {
  pub email: Option<String>,
  pub is_active: Option<bool>,
}

pub async fn update_user(Path(id): Path<Uuid>, State(db): State<DbConnection>, Json(payload): Json<UpdateUserBody>) -> Result<impl IntoResponse, ForgeError> {
  let u = user::Entity::find_by_id(id).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?.ok_or_else(|| ForgeError::Generic("User not found".into()))?;
  let mut am: user::ActiveModel = u.into();
  if let Some(e) = payload.email { let t = e.trim(); if !t.is_empty() && t.contains('@') { am.email = Set(t.to_string()); } }
  if let Some(b) = payload.is_active { am.is_active = Set(b); }
  am.updated_at = Set(chrono::Utc::now().naive_utc());
  am.update(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

pub async fn delete_user(Path(id): Path<Uuid>, State(db): State<DbConnection>) -> Result<impl IntoResponse, ForgeError> {
  membership::Entity::delete_many().filter(membership::Column::UserId.eq(id)).exec(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let r = user::Entity::delete_by_id(id).exec(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  if r.rows_affected == 0 {
    return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "User not found" }))).into_response());
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

pub async fn get_user_organizations(Path(id): Path<Uuid>, State(db): State<DbConnection>) -> Result<impl IntoResponse, ForgeError> {
  let uors = user_org_role::Entity::find().filter(user_org_role::Column::UserId.eq(id)).all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  if uors.is_empty() {
    return Ok(Json(serde_json::json!({ "organizations": [] })).into_response());
  }
  let role_ids: Vec<Uuid> = uors.iter().map(|x| x.role_id).collect();
  let org_ids: Vec<Uuid> = uors.iter().map(|x| x.org_id).collect::<std::collections::HashSet<_>>().into_iter().collect();
  let roles = org_role::Entity::find().filter(org_role::Column::Id.is_in(role_ids)).all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let orgs = organization::Entity::find().filter(organization::Column::Id.is_in(org_ids.clone())).all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let role_map: std::collections::HashMap<Uuid, _> = roles.into_iter().map(|r| (r.id, r)).collect();
  let org_map: std::collections::HashMap<Uuid, _> = orgs.into_iter().map(|o| (o.id, o)).collect();
  let list: Vec<serde_json::Value> = org_ids.into_iter().map(|org_id| {
    let names: Vec<String> = uors.iter().filter(|x| x.org_id == org_id).filter_map(|x| role_map.get(&x.role_id).map(|r| r.name.clone())).collect();
    let name = org_map.get(&org_id).map(|o| o.name.as_str()).unwrap_or("—");
    serde_json::json!({ "org_id": org_id.to_string(), "org_name": name, "roles": names })
  }).collect();
  Ok(Json(serde_json::json!({ "organizations": list })).into_response())
}

pub async fn users_me() -> impl IntoResponse {
  (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "Not authenticated" })))
}

// ---- Organizations ----
pub async fn list_organizations(State(db): State<DbConnection>) -> Result<impl IntoResponse, ForgeError> {
  let rows = organization::Entity::find().order_by_asc(organization::Column::Name).all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let list: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
    "id": r.id.to_string(), "name": r.name, "slug": r.slug,
    "created_at": r.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
    "updated_at": r.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
  })).collect();
  Ok(Json(serde_json::json!({ "organizations": list })).into_response())
}

#[derive(Deserialize)]
pub struct CreateOrganizationBody {
  pub name: String,
  pub slug: Option<String>,
}

pub async fn create_organization(State(db): State<DbConnection>, Json(payload): Json<CreateOrganizationBody>) -> Result<impl IntoResponse, ForgeError> {
  let name = payload.name.trim();
  if name.is_empty() {
    return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "name is required" }))).into_response());
  }
  let slug = payload.slug.as_deref().map(|s| s.trim()).filter(|s| !s.is_empty()).map(String::from).unwrap_or_else(|| slug_from_name(name));
  let now = chrono::Utc::now().naive_utc();
  let id = Uuid::new_v4();
  organization::Entity::insert(organization::ActiveModel {
    id: Set(id), name: Set(name.to_string()), slug: Set(slug.clone()),
    created_at: Set(now), updated_at: Set(now), ..Default::default()
  })
  .exec(&db)
  .await
  .map_err(|e| ForgeError::Generic(e.to_string()))?;
  Ok((StatusCode::CREATED, Json(serde_json::json!({
    "id": id.to_string(), "name": name, "slug": slug,
    "created_at": now.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
    "updated_at": now.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
  }))).into_response())
}

pub async fn get_organization(Path(id): Path<Uuid>, State(db): State<DbConnection>) -> Result<impl IntoResponse, ForgeError> {
  let o = organization::Entity::find_by_id(id).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let Some(o) = o else {
    return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Organization not found" }))).into_response());
  };
  Ok(Json(serde_json::json!({
    "id": o.id.to_string(), "name": o.name, "slug": o.slug,
    "created_at": o.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
    "updated_at": o.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
  })).into_response())
}

#[derive(Deserialize)]
pub struct UpdateOrganizationBody {
  pub name: Option<String>,
  pub slug: Option<String>,
}

pub async fn update_organization(Path(id): Path<Uuid>, State(db): State<DbConnection>, Json(payload): Json<UpdateOrganizationBody>) -> Result<impl IntoResponse, ForgeError> {
  let o = organization::Entity::find_by_id(id).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?.ok_or_else(|| ForgeError::Generic("Organization not found".into()))?;
  let mut am: organization::ActiveModel = o.into();
  if let Some(n) = payload.name { let t = n.trim(); if !t.is_empty() { am.name = Set(t.to_string()); } }
  if let Some(s) = payload.slug { let t = s.trim(); if !t.is_empty() { am.slug = Set(t.to_string()); } }
  am.updated_at = Set(chrono::Utc::now().naive_utc());
  am.update(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let updated = organization::Entity::find_by_id(id).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?.unwrap();
  Ok(Json(serde_json::json!({
    "id": updated.id.to_string(), "name": updated.name, "slug": updated.slug,
    "created_at": updated.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
    "updated_at": updated.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
  })).into_response())
}

pub async fn delete_organization(Path(id): Path<Uuid>, State(db): State<DbConnection>) -> Result<impl IntoResponse, ForgeError> {
  let r = organization::Entity::delete_by_id(id).exec(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  if r.rows_affected == 0 {
    return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Organization not found" }))).into_response());
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

// ---- Organizations/:id/users ----
pub async fn list_org_users(ScopeFromHeaders(scope): ScopeFromHeaders, Extension(db): Extension<DbConnection>) -> Result<impl IntoResponse, ForgeError> {
  let org_id = scope.organization_id;
  let mems = membership::Entity::find().filter(membership::Column::OrgId.eq(org_id)).all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let user_ids: Vec<Uuid> = mems.iter().map(|m| m.user_id).collect();
  if user_ids.is_empty() {
    return Ok(Json(serde_json::json!({ "users": [] })).into_response());
  }
  let users = user::Entity::find().filter(user::Column::Id.is_in(user_ids.clone())).all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let uors = user_org_role::Entity::find().filter(user_org_role::Column::OrgId.eq(org_id)).all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let role_ids: Vec<Uuid> = uors.iter().map(|x| x.role_id).collect::<std::collections::HashSet<_>>().into_iter().collect();
  let roles = org_role::Entity::find().filter(org_role::Column::Id.is_in(role_ids)).all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let role_map: std::collections::HashMap<Uuid, _> = roles.into_iter().map(|r| (r.id, r)).collect();
  let list: Vec<serde_json::Value> = users.into_iter().map(|u| {
    let rolenames: Vec<String> = uors.iter().filter(|x| x.user_id == u.id).filter_map(|x| role_map.get(&x.role_id).map(|r| r.name.clone())).collect();
    serde_json::json!({
      "id": u.id.to_string(), "email": u.email, "is_active": u.is_active,
      "created_at": u.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
      "roles": rolenames,
    })
  }).collect();
  Ok(Json(serde_json::json!({ "users": list })).into_response())
}

#[derive(Deserialize)]
pub struct AddOrgUserBody {
  pub user_id: Option<Uuid>,
  pub email: Option<String>,
  pub password: Option<String>,
}

/// POST /api/organizations/:id/users — add user to org (membership only). Use .../users/:userId/roles to assign roles.
pub async fn add_org_user(ScopeFromHeaders(scope): ScopeFromHeaders, Extension(db): Extension<DbConnection>, Json(payload): Json<AddOrgUserBody>) -> Result<impl IntoResponse, ForgeError> {
  let org_id = scope.organization_id;
  let (user_id, _created) = match (payload.user_id, payload.email, payload.password) {
    (Some(uid), _, _) => {
      let u = user::Entity::find_by_id(uid).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
      if u.is_none() {
        return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "User not found" }))).into_response());
      }
      if membership::Entity::find().filter(membership::Column::UserId.eq(uid)).filter(membership::Column::OrgId.eq(org_id)).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?.is_some() {
        return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "User already in organization" }))).into_response());
      }
      (uid, false)
    }
    (None, Some(email), Some(password)) if email.trim().contains('@') && password.len() >= 8 => {
      let email = email.trim();
      if user::Entity::find().filter(user::Column::Email.eq(email)).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?.is_some() {
        return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "Email already in use" }))).into_response());
      }
      let now = chrono::Utc::now().naive_utc();
      let user_id = Uuid::new_v4();
      let hash = hash_password(&password).map_err(|e| ForgeError::Generic(e.to_string()))?;
      user::Entity::insert(user::ActiveModel {
        id: Set(user_id), email: Set(email.to_string()), password_hash: Set(hash), is_active: Set(true), is_admin: Set(false),
        current_org_id: Set(Some(org_id)), current_role: Set(None), created_at: Set(now), updated_at: Set(now), ..Default::default()
      }).exec(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
      membership::Entity::insert(membership::ActiveModel {
        id: Set(Uuid::new_v4()), user_id: Set(user_id), org_id: Set(org_id), created_at: Set(now), updated_at: Set(now), ..Default::default()
      }).exec(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
      (user_id, true)
    }
    _ => return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "Provide user_id or email+password" }))).into_response()),
  };
  if !membership::Entity::find().filter(membership::Column::UserId.eq(user_id)).filter(membership::Column::OrgId.eq(org_id)).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?.is_some() {
    let now = chrono::Utc::now().naive_utc();
    membership::Entity::insert(membership::ActiveModel {
      id: Set(Uuid::new_v4()), user_id: Set(user_id), org_id: Set(org_id), created_at: Set(now), updated_at: Set(now), ..Default::default()
    }).exec(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  }
  Ok((StatusCode::CREATED, Json(serde_json::json!({ "user_id": user_id.to_string(), "org_id": org_id.to_string() }))).into_response())
}

#[derive(Deserialize)]
pub struct AddOrgUserRolesBody {
  pub role_ids: Vec<Uuid>,
}

/// POST /api/organizations/:id/users/:userId/roles — assign roles to a user in this org.
pub async fn add_org_user_roles(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  Path(user_id): Path<Uuid>,
  Extension(db): Extension<DbConnection>,
  Json(payload): Json<AddOrgUserRolesBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let org_id = scope.organization_id;
  if membership::Entity::find().filter(membership::Column::OrgId.eq(org_id)).filter(membership::Column::UserId.eq(user_id)).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?.is_none() {
    return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "User not in organization" }))).into_response());
  }
  if payload.role_ids.is_empty() {
    return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "role_ids required" }))).into_response());
  }
  for rid in &payload.role_ids {
    let r = org_role::Entity::find_by_id(*rid).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
    if r.map(|r| r.org_id != org_id).unwrap_or(true) {
      return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "role must belong to organization" }))).into_response());
    }
  }
  let now = chrono::Utc::now().naive_utc();
  for rid in &payload.role_ids {
    let exists = user_org_role::Entity::find()
      .filter(user_org_role::Column::UserId.eq(user_id))
      .filter(user_org_role::Column::OrgId.eq(org_id))
      .filter(user_org_role::Column::RoleId.eq(*rid))
      .one(&db)
      .await
      .map_err(|e| ForgeError::Generic(e.to_string()))?;
    if exists.is_none() {
      user_org_role::Entity::insert(user_org_role::ActiveModel {
        id: Set(Uuid::new_v4()), user_id: Set(user_id), org_id: Set(org_id), role_id: Set(*rid), created_at: Set(now), updated_at: Set(now), ..Default::default()
      }).exec(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
    }
  }
  let role_names: Vec<String> = org_role::Entity::find().filter(org_role::Column::Id.is_in(payload.role_ids)).all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?.into_iter().map(|r| r.name).collect();
  Ok((StatusCode::CREATED, Json(serde_json::json!({ "user_id": user_id.to_string(), "org_id": org_id.to_string(), "roles": role_names }))).into_response())
}

pub async fn get_org_user(ScopeFromHeaders(scope): ScopeFromHeaders, Path(user_id): Path<Uuid>, Extension(db): Extension<DbConnection>) -> Result<impl IntoResponse, ForgeError> {
  let org_id = scope.organization_id;
  let mem = membership::Entity::find().filter(membership::Column::OrgId.eq(org_id)).filter(membership::Column::UserId.eq(user_id)).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let Some(_) = mem else {
    return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "User not in organization" }))).into_response());
  };
  let u = user::Entity::find_by_id(user_id).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?.ok_or_else(|| ForgeError::Generic("User not found".into()))?;
  let uors = user_org_role::Entity::find().filter(user_org_role::Column::OrgId.eq(org_id)).filter(user_org_role::Column::UserId.eq(user_id)).all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let role_ids: Vec<Uuid> = uors.iter().map(|x| x.role_id).collect();
  let roles = org_role::Entity::find().filter(org_role::Column::Id.is_in(role_ids)).all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let names: Vec<String> = roles.into_iter().map(|r| r.name).collect();
  Ok(Json(serde_json::json!({
    "id": u.id.to_string(), "email": u.email, "org_id": org_id.to_string(), "roles": names,
  })).into_response())
}

#[derive(Deserialize)]
pub struct UpdateOrgUserBody {
  pub role_ids: Vec<Uuid>,
}

pub async fn update_org_user(ScopeFromHeaders(scope): ScopeFromHeaders, Path(user_id): Path<Uuid>, Extension(db): Extension<DbConnection>, Json(payload): Json<UpdateOrgUserBody>) -> Result<impl IntoResponse, ForgeError> {
  let org_id = scope.organization_id;
  if membership::Entity::find().filter(membership::Column::OrgId.eq(org_id)).filter(membership::Column::UserId.eq(user_id)).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?.is_none() {
    return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "User not in organization" }))).into_response());
  }
  for rid in &payload.role_ids {
    let r = org_role::Entity::find_by_id(*rid).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
    if r.map(|r| r.org_id != org_id).unwrap_or(true) {
      return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "role must belong to organization" }))).into_response());
    }
  }
  user_org_role::Entity::delete_many().filter(user_org_role::Column::OrgId.eq(org_id)).filter(user_org_role::Column::UserId.eq(user_id)).exec(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let now = chrono::Utc::now().naive_utc();
  for rid in &payload.role_ids {
  user_org_role::Entity::insert(user_org_role::ActiveModel {
    id: Set(Uuid::new_v4()), user_id: Set(user_id), org_id: Set(org_id), role_id: Set(*rid), created_at: Set(now), updated_at: Set(now), ..Default::default()
  }).exec(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

pub async fn delete_org_user(ScopeFromHeaders(scope): ScopeFromHeaders, Path(user_id): Path<Uuid>, Extension(db): Extension<DbConnection>) -> Result<impl IntoResponse, ForgeError> {
  let org_id = scope.organization_id;
  let r = membership::Entity::delete_many().filter(membership::Column::OrgId.eq(org_id)).filter(membership::Column::UserId.eq(user_id)).exec(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  user_org_role::Entity::delete_many().filter(user_org_role::Column::OrgId.eq(org_id)).filter(user_org_role::Column::UserId.eq(user_id)).exec(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  if r.rows_affected == 0 {
    return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "User not in organization" }))).into_response());
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

// ---- Organizations/:id/roles ----
pub async fn list_org_roles(ScopeFromHeaders(scope): ScopeFromHeaders, Extension(db): Extension<DbConnection>) -> Result<impl IntoResponse, ForgeError> {
  let org_id = scope.organization_id;
  let rows = org_role::Entity::find().filter(org_role::Column::OrgId.eq(org_id)).order_by_asc(org_role::Column::Name).all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let list: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
    "id": r.id.to_string(), "org_id": r.org_id.to_string(), "name": r.name, "display_name": r.display_name,
    "created_at": r.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
    "updated_at": r.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
  })).collect();
  Ok(Json(serde_json::json!({ "roles": list })).into_response())
}

#[derive(Deserialize)]
pub struct CreateOrgRoleBody {
  pub name: String,
  pub display_name: Option<String>,
}

pub async fn create_org_role(ScopeFromHeaders(scope): ScopeFromHeaders, Extension(db): Extension<DbConnection>, Json(payload): Json<CreateOrgRoleBody>) -> Result<impl IntoResponse, ForgeError> {
  let org_id = scope.organization_id;
  let name = payload.name.trim();
  if name.is_empty() {
    return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "name is required" }))).into_response());
  }
  if org_role::Entity::find().filter(org_role::Column::OrgId.eq(org_id)).filter(org_role::Column::Name.eq(name)).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?.is_some() {
    return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "Role name exists" }))).into_response());
  }
  let now = chrono::Utc::now().naive_utc();
  let id = Uuid::new_v4();
  let display_name = payload.display_name.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()).map(String::from);
  org_role::Entity::insert(org_role::ActiveModel {
    id: Set(id), org_id: Set(org_id), name: Set(name.to_string()), display_name: Set(display_name.clone()),
    created_at: Set(now), updated_at: Set(now), ..Default::default()
  }).exec(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  Ok((StatusCode::CREATED, Json(serde_json::json!({
    "id": id.to_string(), "org_id": org_id.to_string(), "name": name, "display_name": display_name,
    "created_at": now.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
    "updated_at": now.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
  }))).into_response())
}

pub async fn get_org_role(ScopeFromHeaders(scope): ScopeFromHeaders, Path(role_id): Path<Uuid>, Extension(db): Extension<DbConnection>) -> Result<impl IntoResponse, ForgeError> {
  let org_id = scope.organization_id;
  let r = org_role::Entity::find_by_id(role_id).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let Some(r) = r else {
    return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Role not found" }))).into_response());
  };
  if r.org_id != org_id {
    return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Role not in organization" }))).into_response());
  }
  Ok(Json(serde_json::json!({
    "id": r.id.to_string(), "org_id": r.org_id.to_string(), "name": r.name, "display_name": r.display_name,
    "created_at": r.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
    "updated_at": r.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
  })).into_response())
}

#[derive(Deserialize)]
pub struct UpdateOrgRoleBody {
  pub name: Option<String>,
  pub display_name: Option<String>,
}

pub async fn update_org_role(ScopeFromHeaders(scope): ScopeFromHeaders, Path(role_id): Path<Uuid>, Extension(db): Extension<DbConnection>, Json(payload): Json<UpdateOrgRoleBody>) -> Result<impl IntoResponse, ForgeError> {
  let org_id = scope.organization_id;
  let r = org_role::Entity::find_by_id(role_id).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?.ok_or_else(|| ForgeError::Generic("Role not found".into()))?;
  if r.org_id != org_id {
    return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Role not in organization" }))).into_response());
  }
  let mut am: org_role::ActiveModel = r.into();
  if let Some(n) = payload.name { let t = n.trim(); if !t.is_empty() { am.name = Set(t.to_string()); } }
  if let Some(d) = payload.display_name { am.display_name = Set(Some(d.trim().to_string()).filter(|s| !s.is_empty())); }
  am.updated_at = Set(chrono::Utc::now().naive_utc());
  am.update(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

pub async fn delete_org_role(ScopeFromHeaders(scope): ScopeFromHeaders, Path(role_id): Path<Uuid>, Extension(db): Extension<DbConnection>) -> Result<impl IntoResponse, ForgeError> {
  let org_id = scope.organization_id;
  let r = org_role::Entity::find_by_id(role_id).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let Some(r) = r else {
    return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Role not found" }))).into_response());
  };
  if r.org_id != org_id {
    return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Role not in organization" }))).into_response());
  }
  let n = user_org_role::Entity::find().filter(user_org_role::Column::RoleId.eq(role_id)).count(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  if n > 0 {
    return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "Cannot delete role: users have this role" }))).into_response());
  }
  org_role::Entity::delete_by_id(role_id).exec(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

// ---- Organizations/:id/roles/:roleId/permissions ----
pub async fn list_org_role_permissions(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  Path(role_id): Path<Uuid>,
  Extension(db): Extension<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  let org_id = scope.organization_id;
  let r = org_role::Entity::find_by_id(role_id).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let Some(r) = r else {
    return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Role not found" }))).into_response());
  };
  if r.org_id != org_id {
    return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Role not in organization" }))).into_response());
  }
  let rows = role_permission::Entity::find()
    .filter(role_permission::Column::Scope.eq("org"))
    .filter(role_permission::Column::OrgId.eq(org_id))
    .filter(role_permission::Column::RoleName.eq(&r.name))
    .all(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  let keys: Vec<&str> = rows.iter().map(|x| x.permission_key.as_str()).collect();
  Ok(Json(serde_json::json!({ "permissions": keys })).into_response())
}

#[derive(Deserialize)]
pub struct AddOrgRolePermissionBody {
  pub permission_key: String,
}

pub async fn add_org_role_permission(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  Path(role_id): Path<Uuid>,
  Extension(db): Extension<DbConnection>,
  Json(payload): Json<AddOrgRolePermissionBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let org_id = scope.organization_id;
  let r = org_role::Entity::find_by_id(role_id).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let Some(r) = r else {
    return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Role not found" }))).into_response());
  };
  if r.org_id != org_id {
    return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Role not in organization" }))).into_response());
  }
  let key = payload.permission_key.trim();
  if !DASHBOARD_PERMISSIONS.contains(&key) {
    return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "error": "invalid permission_key" }))).into_response());
  }
  let exists = role_permission::Entity::find()
    .filter(role_permission::Column::Scope.eq("org"))
    .filter(role_permission::Column::OrgId.eq(org_id))
    .filter(role_permission::Column::RoleName.eq(&r.name))
    .filter(role_permission::Column::PermissionKey.eq(key))
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if exists.is_none() {
    let id = Uuid::new_v4();
    role_permission::Entity::insert(role_permission::ActiveModel {
      id: Set(id),
      scope: Set("org".to_string()),
      role_name: Set(r.name.clone()),
      permission_key: Set(key.to_string()),
      org_id: Set(Some(org_id)),
      ..Default::default()
    })
    .exec(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  }
  Ok((StatusCode::CREATED, Json(serde_json::json!({ "permission_key": key }))).into_response())
}

#[derive(Deserialize)]
pub struct DeleteOrgRolePermissionBody {
  pub permission_key: String,
}

pub async fn delete_org_role_permission(
  ScopeFromHeaders(scope): ScopeFromHeaders,
  Path(role_id): Path<Uuid>,
  Extension(db): Extension<DbConnection>,
  Json(payload): Json<DeleteOrgRolePermissionBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let org_id = scope.organization_id;
  let r = org_role::Entity::find_by_id(role_id).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let Some(r) = r else {
    return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Role not found" }))).into_response());
  };
  if r.org_id != org_id {
    return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Role not in organization" }))).into_response());
  }
  let key = payload.permission_key.trim();
  let res = role_permission::Entity::delete_many()
    .filter(role_permission::Column::Scope.eq("org"))
    .filter(role_permission::Column::OrgId.eq(org_id))
    .filter(role_permission::Column::RoleName.eq(&r.name))
    .filter(role_permission::Column::PermissionKey.eq(key))
    .exec(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if res.rows_affected == 0 {
    return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Permission not assigned" }))).into_response());
  }
  Ok(Json(serde_json::json!({ "ok": true })).into_response())
}

// ---- Audit ----
#[derive(Deserialize)]
pub struct ListAuditLogQuery {
  pub from: Option<String>,
  pub to: Option<String>,
  pub limit: Option<u64>,
  pub offset: Option<u64>,
}

pub async fn list_audit_log(State(db): State<DbConnection>, axum::extract::Query(q): axum::extract::Query<ListAuditLogQuery>) -> Result<impl IntoResponse, ForgeError> {
  let limit = q.limit.unwrap_or(50).min(200);
  let offset = q.offset.unwrap_or(0);
  let mut query = audit_log::Entity::find();
  if let Some(ref from) = q.from {
    if let Ok(naive) = NaiveDateTime::parse_from_str(from, "%Y-%m-%dT%H:%M:%S%.fZ") {
      query = query.filter(audit_log::Column::OccurredAt.gte(naive));
    } else if let Ok(naive) = NaiveDateTime::parse_from_str(from, "%Y-%m-%d") {
      query = query.filter(audit_log::Column::OccurredAt.gte(naive));
    }
  }
  if let Some(ref to) = q.to {
    if let Ok(naive) = NaiveDateTime::parse_from_str(to, "%Y-%m-%dT%H:%M:%S%.fZ") {
      query = query.filter(audit_log::Column::OccurredAt.lte(naive));
    } else if let Ok(naive) = NaiveDateTime::parse_from_str(to, "%Y-%m-%d") {
      query = query.filter(audit_log::Column::OccurredAt.lt(naive + chrono::Duration::days(1)));
    }
  }
  let total = query.clone().count(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let rows = query.order_by_desc(audit_log::Column::OccurredAt).limit(limit).offset(offset).all(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let entries: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
    "id": r.id.to_string(), "event_kind": r.event_kind, "actor_id": r.actor_id.to_string(),
    "subject_id": r.subject_id.map(|u| u.to_string()), "organization_id": r.organization_id.map(|u| u.to_string()),
    "action": r.action, "resource_type": r.resource_type, "resource_id": r.resource_id.map(|u| u.to_string()),
    "outcome": r.outcome, "reason": r.reason,
    "occurred_at": r.occurred_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
  })).collect();
  Ok(Json(serde_json::json!({ "entries": entries, "total": total })).into_response())
}

pub async fn get_audit_log(Path(id): Path<Uuid>, State(db): State<DbConnection>) -> Result<impl IntoResponse, ForgeError> {
  let r = audit_log::Entity::find_by_id(id).one(&db).await.map_err(|e| ForgeError::Generic(e.to_string()))?;
  let Some(r) = r else {
    return Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Audit log entry not found" }))).into_response());
  };
  Ok(Json(serde_json::json!({
    "id": r.id.to_string(),
    "event_kind": r.event_kind,
    "actor_id": r.actor_id.to_string(),
    "subject_id": r.subject_id.map(|u| u.to_string()),
    "organization_id": r.organization_id.map(|u| u.to_string()),
    "action": r.action,
    "resource_type": r.resource_type,
    "resource_id": r.resource_id.map(|u| u.to_string()),
    "outcome": r.outcome,
    "reason": r.reason,
    "occurred_at": r.occurred_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
  })).into_response())
}
