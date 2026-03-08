//! POST /api/auth/register — create account (email + password).

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use forge_core::Valid;
use forge_db::DbConnection;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use db::models::{api_token, membership, org_role, organization, user, user_org_role};
use forge_auth::token_auth::{hash_api_token, hash_password};

use crate::Error as ForgeError;

#[derive(Deserialize, Validate)]
pub struct RegisterRequest {
  #[validate(email)]
  pub email: String,
  #[validate(length(min = 8))]
  pub password: String,
}

pub async fn register(
  State(db): State<DbConnection>,
  Valid(Json(payload)): Valid<Json<RegisterRequest>>,
) -> Result<impl IntoResponse, ForgeError> {
  tracing::debug!(target: "app::auth", "route: POST /api/auth/register email={}", payload.email);
  let password_hash =
    hash_password(&payload.password).map_err(|e| ForgeError::Generic(e.to_string()))?;
  let now = Utc::now().naive_utc();
  let user_id = Uuid::new_v4();
  let org_id = Uuid::new_v4();
  let membership_id = Uuid::new_v4();

  let tx = db.begin().await?;
  let new_user = user::ActiveModel {
    id: Set(user_id),
    email: Set(payload.email.clone()),
    password_hash: Set(password_hash),
    is_active: Set(true),
    is_admin: Set(false),
    current_org_id: Set(None),
    current_role: Set(None),
    created_at: Set(now),
    updated_at: Set(now),
  };
  user::Entity::insert(new_user).exec(&tx).await?;

  let slug = format!("org-{}", org_id.as_simple());
  let new_org = organization::ActiveModel {
    id: Set(org_id),
    name: Set(format!("{}'s workspace", payload.email)),
    slug: Set(slug),
    created_at: Set(now),
    updated_at: Set(now),
  };
  organization::Entity::insert(new_org).exec(&tx).await?;

  for &name in &["owner", "admin", "editor", "viewer"] {
    let role_id = Uuid::new_v4();
    let r = org_role::ActiveModel {
      id: Set(role_id),
      org_id: Set(org_id),
      name: Set(name.to_string()),
      display_name: Set(None),
      created_at: Set(now),
      updated_at: Set(now),
    };
    org_role::Entity::insert(r).exec(&tx).await?;
  }
  db::seed_role_permissions_for_org(&tx, org_id)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;

  let new_membership = membership::ActiveModel {
    id: Set(membership_id),
    user_id: Set(user_id),
    org_id: Set(org_id),
    created_at: Set(now),
    updated_at: Set(now),
  };
  membership::Entity::insert(new_membership).exec(&tx).await?;

  let owner_role = org_role::Entity::find()
    .filter(org_role::Column::OrgId.eq(org_id))
    .filter(org_role::Column::Name.eq("owner"))
    .one(&tx)
    .await?
    .ok_or_else(|| ForgeError::Generic("org_role owner not found".into()))?;
  let uor_id = Uuid::new_v4();
  let uor = user_org_role::ActiveModel {
    id: Set(uor_id),
    user_id: Set(user_id),
    org_id: Set(org_id),
    role_id: Set(owner_role.id),
    created_at: Set(now),
    updated_at: Set(now),
  };
  user_org_role::Entity::insert(uor).exec(&tx).await?;

  let u = user::Entity::find_by_id(user_id)
    .one(&tx)
    .await?
    .ok_or_else(|| ForgeError::Generic("User not found".into()))?;
  let mut am: user::ActiveModel = u.into();
  am.current_org_id = Set(Some(org_id));
  am.current_role = Set(Some("owner".to_string()));
  am.updated_at = Set(now);
  am.update(&tx).await?;

  tx.commit().await?;

  let secret = format!("forge_{}", Uuid::new_v4().to_string().replace('-', ""));
  let token_hash = hash_api_token(&secret);
  let token_id = Uuid::new_v4();
  let token_model = api_token::ActiveModel {
    id: Set(token_id),
    user_id: Set(user_id),
    token_hash: Set(token_hash),
    name: Set(Some("Registration".to_string())),
    last_used_at: Set(None),
    expires_at: Set(None),
    created_at: Set(now),
    updated_at: Set(now),
  };
  api_token::Entity::insert(token_model).exec(&db).await?;

  Ok(
    (
      StatusCode::CREATED,
      Json(serde_json::json!({ "ok": true, "token": secret })),
    )
      .into_response(),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn register_request_valid_email_and_password_passes() {
    let r = RegisterRequest {
      email: "user@example.com".to_string(),
      password: "password123".to_string(),
    };
    assert!(r.validate().is_ok());
  }

  #[test]
  fn register_request_invalid_email_fails() {
    let r = RegisterRequest {
      email: "not-an-email".to_string(),
      password: "password123".to_string(),
    };
    assert!(r.validate().is_err());
  }

  #[test]
  fn register_request_short_password_fails() {
    let r = RegisterRequest {
      email: "user@example.com".to_string(),
      password: "short".to_string(),
    };
    assert!(r.validate().is_err());
  }
}
