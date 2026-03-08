//! GET /api/auth/profiles — list (org, role) profiles for scope picker.

use axum::{Json, extract::State, response::IntoResponse};
use forge_auth::token_auth::RequireAuth;
use forge_db::DbConnection;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use db::auth::Backend;
use db::models::{org_role, organization, user, user_org_role};

use crate::Error as ForgeError;

pub async fn profiles_list(
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  let uors = user_org_role::Entity::find()
    .filter(user_org_role::Column::UserId.eq(user.id))
    .all(&db)
    .await?;
  if uors.is_empty() {
    return Ok(Json(serde_json::json!({ "profiles": [] })).into_response());
  }
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
  let profiles: Vec<serde_json::Value> = uors
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
    .collect();
  Ok(Json(serde_json::json!({ "profiles": profiles })).into_response())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn profiles_list_handler_exists() {
    let _ = profiles_list;
  }
}
