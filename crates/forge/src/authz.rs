pub use forge_authz::*;

use async_trait::async_trait;
use axum_login::{AuthSession, AuthnBackend};
use sea_orm::ConnectionTrait;

use crate::audit::{AuditEvent, EventKind, Outcome};

// AuthzContext is implemented for AuthSession in forge-authz directly.

/// Extension trait for AuthSession to provide authorization guards.
#[async_trait]
pub trait AuthSessionGuardExt {
  /// Guard a handler by requiring a specific action and role.
  fn guard(&self, action: Action, role: Role) -> Result<(), AuthzError>;

  /// Guard and record the decision in the audit log (authz event, allowed/denied).
  async fn guard_and_audit(
    &self,
    db: &impl ConnectionTrait,
    action: Action,
    role: Role,
    resource_type: &str,
    resource_id: Option<uuid::Uuid>,
  ) -> Result<(), AuthzError>;
}

#[async_trait]
impl<B> AuthSessionGuardExt for AuthSession<B>
where
  B: AuthnBackend + Send + Sync,
  B::User: AuthzContext + Send,
{
  fn guard(&self, _action: Action, role: Role) -> Result<(), AuthzError> {
    if self.user.is_none() {
      return Err(AuthzError::Forbidden);
    }

    // Require the session user to have the given role (or higher) in the current org.
    match self.role() {
      Some(user_role) if user_role == role => Ok(()),
      Some(Role::Owner) => Ok(()), // Owner can satisfy any role check
      Some(Role::Admin) if matches!(role, Role::Admin | Role::Editor | Role::Viewer) => Ok(()),
      Some(Role::Editor) if matches!(role, Role::Editor | Role::Viewer) => Ok(()),
      Some(Role::Viewer) if role == Role::Viewer => Ok(()),
      _ => Err(AuthzError::Forbidden),
    }
  }

  async fn guard_and_audit(
    &self,
    db: &impl ConnectionTrait,
    action: Action,
    role: Role,
    resource_type: &str,
    resource_id: Option<uuid::Uuid>,
  ) -> Result<(), AuthzError> {
    let result = self.guard(action, role);
    let outcome = if result.is_ok() {
      Outcome::Allowed
    } else {
      Outcome::Denied
    };
    let event = AuditEvent {
      event_kind: EventKind::Authz,
      actor_id: self.requester_id(),
      subject_id: Some(self.subject_id()),
      organization_id: self.organization_id(),
      action,
      resource_type: resource_type.to_string(),
      resource_id,
      outcome,
      reason: None,
    };
    let _ = crate::audit::log(db, event).await;
    result
  }
}

/// Guard and audit for a concrete user (e.g. from [crate::token_auth::RequireAuth]). Use when the
/// handler has the user from token or session and needs to enforce role and record the decision.
pub async fn guard_and_audit_user<U: AuthzContext + Send>(
  user: &U,
  db: &impl ConnectionTrait,
  action: Action,
  role: Role,
  resource_type: &str,
  resource_id: Option<uuid::Uuid>,
) -> Result<(), AuthzError> {
  let result = guard_user(user, action, role);
  let outcome = if result.is_ok() {
    Outcome::Allowed
  } else {
    Outcome::Denied
  };
  let event = AuditEvent {
    event_kind: EventKind::Authz,
    actor_id: user.requester_id(),
    subject_id: Some(user.subject_id()),
    organization_id: user.organization_id(),
    action,
    resource_type: resource_type.to_string(),
    resource_id,
    outcome,
    reason: None,
  };
  let _ = crate::audit::log(db, event).await;
  result
}

/// Guard by role for a concrete user (no audit). Use with [guard_and_audit_user] for audit.
fn guard_user<U: AuthzContext>(user: &U, _action: Action, role: Role) -> Result<(), AuthzError> {
  match user.role() {
    Some(user_role) if user_role == role => Ok(()),
    Some(Role::Owner) => Ok(()),
    Some(Role::Admin) if matches!(role, Role::Admin | Role::Editor | Role::Viewer) => Ok(()),
    Some(Role::Editor) if matches!(role, Role::Editor | Role::Viewer) => Ok(()),
    Some(Role::Viewer) if role == Role::Viewer => Ok(()),
    _ => Err(AuthzError::Forbidden),
  }
}

/// Record an authz denied event for unauthenticated or unauthorized access (e.g. before returning 401).
pub async fn record_authz_denied(
  db: &impl ConnectionTrait,
  action: Action,
  resource_type: &str,
  resource_id: Option<uuid::Uuid>,
) {
  let event = AuditEvent {
    event_kind: EventKind::Authz,
    actor_id: uuid::Uuid::nil(),
    subject_id: Some(uuid::Uuid::nil()),
    organization_id: None,
    action,
    resource_type: resource_type.to_string(),
    resource_id,
    outcome: Outcome::Denied,
    reason: None,
  };
  let _ = crate::audit::log(db, event).await;
}
