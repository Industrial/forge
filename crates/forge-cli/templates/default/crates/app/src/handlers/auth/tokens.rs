//! POST /api/auth/tokens — create API token (Bearer) for machine access.

use axum::{Json, response::IntoResponse};
use chrono::Utc;
use forge_core::Valid;
use sea_orm::{EntityTrait, Set};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use db::models::api_token;
use forge_auth::token_auth::hash_api_token;

use crate::Error as ForgeError;

use super::shared::DbFromScope;
use db::auth::Backend;
use db::models::user;
use forge_auth::token_auth::RequireAuth;

#[derive(Deserialize, Validate)]
pub struct CreateTokenRequest {
  pub name: Option<String>,
}

pub async fn create_token(
  auth: RequireAuth<Backend, user::Model>,
  DbFromScope(db): DbFromScope,
  Valid(Json(payload)): Valid<Json<CreateTokenRequest>>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = &auth.0;
  tracing::debug!(target: "app::auth", "route: POST /api/auth/tokens user_id={}", user.id);
  let secret = format!("forge_{}", Uuid::new_v4().to_string().replace('-', ""));
  let token_hash = hash_api_token(&secret);
  let now = Utc::now().naive_utc();
  let id = Uuid::new_v4();
  let model = api_token::ActiveModel {
    id: Set(id),
    user_id: Set(user.id),
    token_hash: Set(token_hash),
    name: Set(payload.name),
    last_used_at: Set(None),
    expires_at: Set(None),
    created_at: Set(now),
    updated_at: Set(now),
  };
  api_token::Entity::insert(model).exec(&db).await?;
  Ok(Json(serde_json::json!({ "token": secret })))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn create_token_request_accepts_empty_name() {
    let r = CreateTokenRequest { name: None };
    assert!(r.validate().is_ok());
  }

  #[test]
  fn create_token_request_accepts_some_name() {
    let r = CreateTokenRequest {
      name: Some("my-token".to_string()),
    };
    assert!(r.validate().is_ok());
  }
}
