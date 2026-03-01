use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Redirect},
  Json,
};
use axum_login::AuthSession;
use chrono::Utc;
use forge::auth::{hash_api_token, hash_password};
use forge::audit::{AuditEvent, EventKind, Outcome};
use forge::authz::{guard_and_audit_user, Action, AuthzContext, Role};
use forge::validation::Valid;
use forge::token_auth::{OptionalRequireAuth, RequireAuth};
use forge::authz::record_authz_denied;
use forge::{DbConnection, Error as ForgeError};
use sea_orm::{ActiveModelTrait, EntityTrait, Set, TransactionTrait};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use db::auth::Backend;
use db::models::{api_token, organization, membership, user};

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
  let password_hash = hash_password(&payload.password)?;
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

  let new_membership = membership::ActiveModel {
    id: Set(membership_id),
    user_id: Set(user_id),
    org_id: Set(org_id),
    role: Set("owner".to_string()),
    created_at: Set(now),
    updated_at: Set(now),
  };
  membership::Entity::insert(new_membership).exec(&tx).await?;

  let u = user::Entity::find_by_id(user_id).one(&tx).await?.ok_or_else(|| ForgeError::Generic("User not found".into()))?;
  let mut am: user::ActiveModel = u.into();
  am.current_org_id = Set(Some(org_id));
  am.current_role = Set(Some("owner".to_string()));
  am.updated_at = Set(now);
  am.update(&tx).await?;

  tx.commit().await?;

  Ok((StatusCode::CREATED, "User registered successfully"))
}

#[derive(Deserialize, Validate)]
pub struct LoginRequest {
  #[validate(email)]
  pub email: String,
  #[validate(length(min = 1))]
  pub password: String,
}

pub async fn login(
  mut auth_session: AuthSession<Backend>,
  State(db): State<DbConnection>,
  Valid(Json(payload)): Valid<Json<LoginRequest>>,
) -> Result<impl IntoResponse, ForgeError> {
  let credentials = db::auth::Credentials {
    email: payload.email,
    password: payload.password,
  };

  let user = auth_session
    .authenticate(credentials)
    .await
    .map_err(|e| ForgeError::Generic(format!("Authentication error: {}", e)))?;

  if let Some(ref user) = user {
    auth_session
      .login(user)
      .await
      .map_err(|e| ForgeError::Generic(format!("Login error: {}", e)))?;
    let _ = forge::audit::log(
      &db,
      AuditEvent {
        event_kind: EventKind::Auth,
        actor_id: user.id,
        subject_id: Some(user.id),
        organization_id: user.current_org_id,
        action: Action::Manage,
        resource_type: "auth".to_string(),
        resource_id: None,
        outcome: Outcome::Success,
        reason: Some("login".to_string()),
      },
    )
    .await;
    Ok(Redirect::to("/dashboard").into_response())
  } else {
    let _ = forge::audit::log(
      &db,
      AuditEvent {
        event_kind: EventKind::Auth,
        actor_id: Uuid::nil(),
        subject_id: None,
        organization_id: None,
        action: Action::Manage,
        resource_type: "auth".to_string(),
        resource_id: None,
        outcome: Outcome::Failure,
        reason: Some("failed_login".to_string()),
      },
    )
    .await;
    Ok((StatusCode::UNAUTHORIZED, "Invalid credentials").into_response())
  }
}

pub async fn logout(
  mut auth_session: AuthSession<Backend>,
  State(db): State<DbConnection>,
) -> impl IntoResponse {
  let actor_id = auth_session.requester_id();
  let org_id = auth_session.organization_id();
  auth_session.logout().await.unwrap();
  let _ = forge::audit::log(
    &db,
    AuditEvent {
      event_kind: EventKind::Auth,
      actor_id,
      subject_id: Some(actor_id),
      organization_id: org_id,
      action: Action::Manage,
      resource_type: "auth".to_string(),
      resource_id: None,
      outcome: Outcome::Success,
      reason: Some("logout".to_string()),
    },
  )
  .await;
  StatusCode::OK
}

pub async fn profile(auth_session: AuthSession<Backend>) -> impl IntoResponse {
  match &auth_session.user {
    Some(user) => {
      let org_id = auth_session.organization_id().expect("profile requires organization_id");
      let role = auth_session.role().expect("profile requires role");
      format!("Hello, {}! org={} role={:?}", user.email, org_id, role).into_response()
    }
    None => (StatusCode::UNAUTHORIZED, "Not logged in").into_response(),
  }
}

#[derive(Deserialize, Validate)]
pub struct CreateTokenRequest {
  pub name: Option<String>,
}

pub async fn create_token(
  RequireAuth(user): RequireAuth<Backend>,
  State(db): State<DbConnection>,
  Valid(Json(payload)): Valid<Json<CreateTokenRequest>>,
) -> Result<impl IntoResponse, ForgeError> {
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
  Ok(Json(forge::serde_json::json!({ "token": secret })))
}

/// Shallow Gate example: only users with Role::Owner (or Admin) can access. Audits the decision.
pub async fn admin_only(
  OptionalRequireAuth(maybe_user): OptionalRequireAuth<Backend>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  let user = match maybe_user {
    Some(u) => u,
    None => {
      record_authz_denied(&db, Action::Manage, "admin", None).await;
      return Ok((StatusCode::UNAUTHORIZED, "Authentication required").into_response());
    }
  };
  guard_and_audit_user(&user, &db, Action::Manage, Role::Owner, "admin", None).await?;
  Ok((StatusCode::OK, format!("Admin only: access granted for {}", user.email)).into_response())
}
