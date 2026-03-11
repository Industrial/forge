//! GET/POST /api/auth/organizations, PATCH/DELETE /api/auth/organizations/{id}.

use axum::{
  Json,
  extract::{Extension, Path},
  http::StatusCode,
  response::IntoResponse,
};
use forge_auth::token_auth::RequireAuth;
use forge_live::{Channel, InMemoryLiveBackend, LiveEvent, broadcast_to_channel};
use sea_orm::{EntityTrait, QueryOrder};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use db::auth::Backend;
use db::models::{org_role, organization, user};
use db::organization::{
  CreateOrganizationBody, UpdateOrganizationBody as DbUpdateOrganizationBody,
  create_organization_impl, update_organization_impl,
};

use crate::Error as ForgeError;

use super::shared::{
  DbFromScope, PERMISSION_ORGS_READ, PERMISSION_ORGS_WRITE, ScopeFromHeaders, require_permission,
};

/// GET /api/auth/organizations — list organizations. Requires organization.read.
pub async fn list_organizations(
  ScopeFromHeaders(scope, _): ScopeFromHeaders<user::Model>,
  auth: RequireAuth<Backend, user::Model>,
  DbFromScope(db): DbFromScope,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_ORGS_READ, Some(&scope)).await {
    return Ok(resp);
  }
  let rows = organization::Entity::find()
    .order_by_asc(organization::Column::Name)
    .all(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  let list: Vec<serde_json::Value> = rows
    .into_iter()
    .map(|r| {
      serde_json::json!({
        "id": r.id.to_string(),
        "name": r.name,
        "slug": r.slug,
        "created_at": r.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
        "updated_at": r.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
      })
    })
    .collect();
  Ok(Json(serde_json::json!({ "organizations": list })).into_response())
}

/// POST /api/auth/organizations — create organization. Requires organization.create.
pub async fn create_organization(
  ScopeFromHeaders(scope, _): ScopeFromHeaders<user::Model>,
  auth: RequireAuth<Backend, user::Model>,
  DbFromScope(db): DbFromScope,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Json(payload): Json<CreateOrganizationBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_ORGS_WRITE, Some(&scope)).await {
    return Ok(resp);
  }
  let id = create_organization_impl(&db, &payload)
    .await
    .map_err(crate::Error::from)?;
  let o = organization::Entity::find_by_id(id)
    .one(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?
    .ok_or_else(|| ForgeError::Generic("created organization not found".into()))?;
  let now = o.updated_at;
  for &role_name in &["owner", "admin", "editor", "viewer"] {
    let role_id = Uuid::new_v4();
    let r = org_role::ActiveModel {
      id: sea_orm::ActiveValue::Set(role_id),
      org_id: sea_orm::ActiveValue::Set(id),
      name: sea_orm::ActiveValue::Set(role_name.to_string()),
      display_name: sea_orm::ActiveValue::Set(None),
      created_at: sea_orm::ActiveValue::Set(now),
      updated_at: sea_orm::ActiveValue::Set(now),
    };
    org_role::Entity::insert(r)
      .exec(&db)
      .await
      .map_err(|e| ForgeError::Generic(e.to_string()))?;
  }
  db::seed_role_permissions_for_org(&db, id)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if let Some(ref backend) = live_backend {
    let _ = broadcast_to_channel(
      backend,
      &Channel::raw("organizations"),
      &LiveEvent::ResourceChanged {
        resource: "organizations".into(),
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
        "id": o.id.to_string(),
        "name": o.name,
        "slug": o.slug,
        "created_at": o.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
        "updated_at": o.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
      })),
    )
      .into_response(),
  )
}

#[derive(Deserialize)]
pub struct UpdateOrganizationBody {
  pub name: Option<String>,
  pub slug: Option<String>,
}

/// PATCH /api/auth/organizations/{id} — update organization. Requires organization.update.
pub async fn update_organization(
  ScopeFromHeaders(scope, _): ScopeFromHeaders<user::Model>,
  auth: RequireAuth<Backend, user::Model>,
  DbFromScope(db): DbFromScope,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Path(id): Path<Uuid>,
  Json(payload): Json<UpdateOrganizationBody>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_ORGS_WRITE, Some(&scope)).await {
    return Ok(resp);
  }
  let db_payload = DbUpdateOrganizationBody {
    name: payload.name.clone(),
    slug: payload.slug.clone(),
  };
  let updated = update_organization_impl(&db, id, &db_payload)
    .await
    .map_err(crate::Error::from)?;
  if let Some(ref backend) = live_backend {
    let _ = broadcast_to_channel(
      backend,
      &Channel::raw("organizations"),
      &LiveEvent::ResourceChanged {
        resource: "organizations".into(),
        id,
        action: Some("updated".into()),
      },
    )
    .await;
  }
  Ok(
    Json(serde_json::json!({
      "id": updated.id.to_string(),
      "name": updated.name,
      "slug": updated.slug,
      "created_at": updated.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
      "updated_at": updated.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
    }))
    .into_response(),
  )
}

/// DELETE /api/auth/organizations/{id}. Requires organization.delete.
pub async fn delete_organization(
  ScopeFromHeaders(scope, _): ScopeFromHeaders<user::Model>,
  auth: RequireAuth<Backend, user::Model>,
  DbFromScope(db): DbFromScope,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  if let Some(resp) = require_permission(user, &db, PERMISSION_ORGS_WRITE, Some(&scope)).await {
    return Ok(resp);
  }
  let result = organization::Entity::delete_by_id(id)
    .exec(&db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  if result.rows_affected == 0 {
    return Ok(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Organization not found" })),
      )
        .into_response(),
    );
  }
  if let Some(ref backend) = live_backend {
    let _ = broadcast_to_channel(
      backend,
      &Channel::raw("organizations"),
      &LiveEvent::ResourceChanged {
        resource: "organizations".into(),
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
  fn list_organizations_handler_exists() {
    let _ = list_organizations;
  }
}
