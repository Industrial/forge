//! POST /api/auth/login — session login (returns token; client stores it).

use axum::{
  Json,
  extract::{Extension, State},
  http::StatusCode,
  response::IntoResponse,
};
use chrono::Utc;
use forge_db::DbConnection;
use forge_live::InMemoryLiveBackend;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use axum_login::AuthnBackend;
use db::auth::{Backend, Credentials};
use db::models::{api_token, user_org_role};
use forge_audit::{AuditEvent, EventKind, Outcome, log};
use forge_auth::Action;
use forge_auth::token_auth::hash_api_token;

use crate::Error as ForgeError;

use super::shared::{JsonOrForm, broadcast_audit_entry};

#[derive(Deserialize, Validate)]
pub struct LoginRequest {
  #[validate(email)]
  pub email: String,
  #[validate(length(min = 1))]
  pub password: String,
}

pub async fn login(
  State(db): State<DbConnection>,
  Extension(live_backend): Extension<Option<Arc<InMemoryLiveBackend>>>,
  JsonOrForm(payload): JsonOrForm<LoginRequest>,
) -> Result<impl IntoResponse, ForgeError> {
  let email = payload.email.clone();
  tracing::debug!(target: "app::auth", "route: POST /api/auth/login email={}", email);
  payload
    .validate()
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  let credentials = Credentials {
    email: payload.email,
    password: payload.password,
  };

  let backend = Backend::new(db.clone());
  let user = backend
    .authenticate(credentials)
    .await
    .map_err(|e| ForgeError::Generic(format!("Authentication error: {}", e)))?;

  if let Some(ref user) = user {
    let uors = user_org_role::Entity::find()
      .filter(user_org_role::Column::UserId.eq(user.id))
      .all(&db)
      .await
      .unwrap_or_default();
    let needs_scope_select = uors.len() != 1;

    let secret = format!("forge_{}", Uuid::new_v4().to_string().replace('-', ""));
    let token_hash = hash_api_token(&secret);
    let now = Utc::now().naive_utc();
    let token_id = Uuid::new_v4();
    let token_model = api_token::ActiveModel {
      id: Set(token_id),
      user_id: Set(user.id),
      token_hash: Set(token_hash),
      name: Set(Some("Login".to_string())),
      last_used_at: Set(None),
      expires_at: Set(None),
      created_at: Set(now),
      updated_at: Set(now),
    };
    api_token::Entity::insert(token_model).exec(&db).await?;

    let event = AuditEvent {
      event_kind: EventKind::Auth,
      actor_id: user.id,
      subject_id: Some(user.id),
      organization_id: if uors.len() == 1 {
        Some(uors[0].org_id)
      } else {
        None
      },
      action: Action::Manage,
      resource_type: "auth".to_string(),
      resource_id: None,
      outcome: Outcome::Success,
      reason: Some("login".to_string()),
    };
    if let Ok(result) = log(&db, event.clone()).await {
      broadcast_audit_entry(live_backend.as_ref(), &event, &result).await;
    }
    Ok(
      (
        StatusCode::OK,
        Json(serde_json::json!({
          "ok": true,
          "token": secret,
          "needs_scope_select": needs_scope_select
        })),
      )
        .into_response(),
    )
  } else {
    tracing::debug!(target: "app::auth", "login failed: invalid credentials email={}", email);
    let event = AuditEvent {
      event_kind: EventKind::Auth,
      actor_id: Uuid::nil(),
      subject_id: None,
      organization_id: None,
      action: Action::Manage,
      resource_type: "auth".to_string(),
      resource_id: None,
      outcome: Outcome::Failure,
      reason: Some("failed_login".to_string()),
    };
    if let Ok(result) = log(&db, event.clone()).await {
      broadcast_audit_entry(live_backend.as_ref(), &event, &result).await;
    }
    Ok(
      (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({ "error": "Invalid email or password" })),
      )
        .into_response(),
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn login_request_valid_passes() {
    let r = LoginRequest {
      email: "user@example.com".to_string(),
      password: "anything".to_string(),
    };
    assert!(r.validate().is_ok());
  }

  #[test]
  fn login_request_invalid_email_fails() {
    let r = LoginRequest {
      email: "bad".to_string(),
      password: "x".to_string(),
    };
    assert!(r.validate().is_err());
  }

  #[test]
  fn login_request_empty_password_fails() {
    let r = LoginRequest {
      email: "user@example.com".to_string(),
      password: String::new(),
    };
    assert!(r.validate().is_err());
  }
}
