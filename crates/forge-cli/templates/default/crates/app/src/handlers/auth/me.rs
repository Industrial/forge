//! GET /api/auth/me — current user, scopes, permissions (from optional scope headers).

use axum::{Json, extract::State, http::Request, response::IntoResponse};
use forge_auth::token_auth::RequireAuth;
use forge_db::DbConnection;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use db::auth::Backend;
use db::models::{org_role, organization, user, user_org_role};

use crate::Error as ForgeError;

use super::shared::{get_scope_from_headers_map, resolve_permissions};

pub async fn get_me(
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  req: Request<axum::body::Body>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  let scope = get_scope_from_headers_map(req.headers(), user, &db).await;
  let permissions = resolve_permissions(&db, user, scope.as_ref()).await;
  let uors = user_org_role::Entity::find()
    .filter(user_org_role::Column::UserId.eq(user.id))
    .all(&db)
    .await?;
  let scopes: Vec<serde_json::Value> = if uors.is_empty() {
    vec![]
  } else {
    let role_ids: Vec<uuid::Uuid> = uors.iter().map(|u| u.role_id).collect();
    let roles = org_role::Entity::find()
      .filter(org_role::Column::Id.is_in(role_ids))
      .all(&db)
      .await?;
    let org_ids: Vec<uuid::Uuid> = uors
      .iter()
      .map(|u| u.org_id)
      .collect::<std::collections::HashSet<_>>()
      .into_iter()
      .collect();
    let orgs = organization::Entity::find()
      .filter(organization::Column::Id.is_in(org_ids))
      .all(&db)
      .await?;
    let org_map: std::collections::HashMap<uuid::Uuid, organization::Model> =
      orgs.into_iter().map(|o| (o.id, o)).collect();
    let role_map: std::collections::HashMap<uuid::Uuid, org_role::Model> =
      roles.into_iter().map(|r| (r.id, r)).collect();
    uors
      .into_iter()
      .filter_map(|u| {
        let r = role_map.get(&u.role_id)?;
        let o = org_map.get(&u.org_id)?;
        Some(serde_json::json!({
          "org_id": u.org_id.to_string(),
          "org_name": o.name,
          "role_id": u.role_id.to_string(),
          "role": r.name,
        }))
      })
      .collect()
  };
  let needs_scope_select = scopes.len() != 1;
  Ok(Json(serde_json::json!({
    "user": { "id": user.id.to_string(), "email": user.email },
    "scopes": scopes,
    "permissions": permissions,
    "flash": serde_json::Value::Null,
    "needs_scope_select": needs_scope_select,
  })))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn get_me_handler_exists() {
    // Compilation and symbol test
    let _ = get_me;
  }
}
