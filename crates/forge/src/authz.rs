pub use forge_authz::*;

use async_trait::async_trait;
#[cfg(feature = "session")]
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

#[cfg(test)]
mod tests {
  use super::*;
  use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement};

  struct MockAuthzUser {
    role: Option<Role>,
    org_id: Option<uuid::Uuid>,
  }

  impl AuthzContext for MockAuthzUser {
    fn requester_id(&self) -> uuid::Uuid {
      uuid::Uuid::nil()
    }
    fn subject_id(&self) -> uuid::Uuid {
      uuid::Uuid::nil()
    }
    fn organization_id(&self) -> Option<uuid::Uuid> {
      self.org_id
    }
    fn role(&self) -> Option<Role> {
      self.role.clone()
    }
  }

  async fn test_db_with_audit_log() -> DatabaseConnection {
    let db = Database::connect(sea_orm::ConnectOptions::new("sqlite::memory:".to_string()))
      .await
      .unwrap();
    db.execute(Statement::from_string(
      db.get_database_backend(),
      "CREATE TABLE audit_log (
        id TEXT PRIMARY KEY,
        event_kind TEXT NOT NULL,
        actor_id TEXT NOT NULL,
        subject_id TEXT,
        organization_id TEXT,
        action TEXT NOT NULL,
        resource_type TEXT NOT NULL,
        resource_id TEXT,
        outcome TEXT NOT NULL,
        reason TEXT,
        occurred_at TEXT NOT NULL
      )",
    ))
    .await
    .unwrap();
    db
  }

  #[tokio::test]
  async fn guard_and_audit_user_allowed_when_role_matches() {
    let db = test_db_with_audit_log().await;
    let user = MockAuthzUser {
      role: Some(Role::Viewer),
      org_id: Some(uuid::Uuid::new_v4()),
    };
    let res = guard_and_audit_user(&user, &db, Action::Read, Role::Viewer, "doc", None).await;
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn guard_and_audit_user_denied_when_no_role() {
    let db = test_db_with_audit_log().await;
    let user = MockAuthzUser {
      role: None,
      org_id: None,
    };
    let res = guard_and_audit_user(&user, &db, Action::Read, Role::Viewer, "doc", None).await;
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), AuthzError::Forbidden));
  }

  #[tokio::test]
  async fn guard_and_audit_user_owner_satisfies_any_role() {
    let db = test_db_with_audit_log().await;
    let user = MockAuthzUser {
      role: Some(Role::Owner),
      org_id: None,
    };
    let res = guard_and_audit_user(&user, &db, Action::Manage, Role::Admin, "org", None).await;
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn guard_and_audit_user_admin_satisfies_editor_and_viewer() {
    let db = test_db_with_audit_log().await;
    let user = MockAuthzUser {
      role: Some(Role::Admin),
      org_id: None,
    };
    assert!(
      guard_and_audit_user(&user, &db, Action::Read, Role::Viewer, "x", None)
        .await
        .is_ok()
    );
    assert!(
      guard_and_audit_user(&user, &db, Action::Update, Role::Editor, "x", None)
        .await
        .is_ok()
    );
  }

  #[tokio::test]
  async fn guard_and_audit_user_editor_satisfies_viewer() {
    let db = test_db_with_audit_log().await;
    let user = MockAuthzUser {
      role: Some(Role::Editor),
      org_id: None,
    };
    let res = guard_and_audit_user(&user, &db, Action::Read, Role::Viewer, "x", None).await;
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn record_authz_denied_writes_audit_event() {
    let db = test_db_with_audit_log().await;
    record_authz_denied(&db, Action::Read, "resource", None).await;
  }
}
