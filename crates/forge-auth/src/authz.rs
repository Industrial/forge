//! Authorization: context, roles, actions, and policy traits.
//!
//! Implements a "Shallow Gate + Deep Scope" strategy for high-performance
//! SaaS applications using SeaORM and Axum.

use async_trait::async_trait;
use sea_orm::{EntityTrait, Select};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// The intent being performed on a resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
  Read,
  Create,
  Update,
  Delete,
  Manage,
}

/// A scoped role (e.g., Admin within an Organization).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
  Owner,
  Admin,
  Editor,
  Viewer,
  Custom(String),
}

/// Authorization errors.
#[derive(Error, Debug)]
pub enum AuthzError {
  #[error("Forbidden: You do not have permission to perform this action.")]
  Forbidden,

  #[error("Not Found: The resource does not exist or you do not have access.")]
  NotFound,

  #[error("Database error: {0}")]
  DatabaseError(#[from] sea_orm::DbErr),
}

/// The context of the authorization request.
/// Distinguishes between who is making the request (the authenticated user) and who they are acting as.
pub trait AuthzContext {
  type RequesterId: Into<Uuid>;
  type SubjectId: Into<Uuid>;

  /// The ID of the authenticated requester (the user making the request).
  fn requester_id(&self) -> Self::RequesterId;

  /// The ID of the identity context (Subject).
  fn subject_id(&self) -> Self::SubjectId;

  /// Optional organization/tenant context.
  fn organization_id(&self) -> Option<Uuid>;

  /// Optional role in the current organization (for Shallow Gate).
  /// Default is None; implement to enable role-based guards.
  fn role(&self) -> Option<Role> {
    None
  }
}

/// The authorization scope derived from request headers, e.g., `X-Organization-Id` and `X-Role-Id`.
/// Set by middleware, and used to determine current organization and role context for authorization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestScope {
  pub organization_id: Uuid,
  pub role: Role,
}

impl AuthzContext for RequestScope {
  type RequesterId = Uuid;
  type SubjectId = Uuid;

  fn requester_id(&self) -> Self::RequesterId {
    Uuid::nil()
  }

  fn subject_id(&self) -> Self::SubjectId {
    Uuid::nil()
  }

  fn organization_id(&self) -> Option<Uuid> {
    Some(self.organization_id)
  }

  fn role(&self) -> Option<Role> {
    Some(self.role.clone())
  }
}

/// Trait for implicitly scoping SeaORM queries (the "Deep Scope" primitive).
pub trait ForgeScoped<E: EntityTrait> {
  /// Apply authorization filters to a SeaORM query.
  fn scoped<C: AuthzContext>(self, context: &C) -> Select<E>;
}

/// Trait for explicit ReBAC policy logic (the "Logic" primitive).
#[async_trait]
pub trait ForgePolicy<R> {
  /// Check if the requester can perform an action on a resource.
  async fn can<C: AuthzContext>(
    &self,
    context: &C,
    action: Action,
    resource: &R,
    db: &impl sea_orm::ConnectionTrait,
  ) -> Result<bool, AuthzError>;
}

// --- Guard helpers (role-based; audit integration lives in forge-audit) ---

use crate::token_auth::TokenUser;

/// Extension trait for [TokenUser] to provide role-based authorization guards (no audit).
pub trait TokenUserGuardExt<U> {
  /// Guard a handler by requiring a specific action and role. Does not write to the audit log.
  fn guard(&self, action: Action, role: Role) -> Result<(), AuthzError>;
}

impl<U> TokenUserGuardExt<U> for TokenUser<U>
where
  U: AuthzContext<RequesterId = Uuid, SubjectId = Uuid> + Send + Sync,
{
  fn guard(&self, _action: Action, role: Role) -> Result<(), AuthzError> {
    match self.role() {
      Some(user_role) if user_role == role => Ok(()),
      Some(Role::Owner) => Ok(()),
      Some(Role::Admin) if matches!(role, Role::Admin | Role::Editor | Role::Viewer) => Ok(()),
      Some(Role::Editor) if matches!(role, Role::Editor | Role::Viewer) => Ok(()),
      Some(Role::Viewer) if role == Role::Viewer => Ok(()),
      _ => Err(AuthzError::Forbidden),
    }
  }
}

/// Guard by role for a concrete user (no audit). Use with [forge_audit::guard_and_audit_user] for audit.
pub fn guard_user<U: AuthzContext<RequesterId = Uuid, SubjectId = Uuid>>(
  user: &U,
  _action: Action,
  role: Role,
  request_scope: &Option<RequestScope>,
) -> Result<(), AuthzError> {
  let user_role = request_scope
    .as_ref()
    .map(|scope| scope.role.clone())
    .or_else(|| user.role());

  match user_role {
    Some(user_role) if user_role == role => Ok(()),
    Some(Role::Owner) => Ok(()),
    Some(Role::Admin) if matches!(role, Role::Admin | Role::Editor | Role::Viewer) => Ok(()),
    Some(Role::Editor) if matches!(role, Role::Editor | Role::Viewer) => Ok(()),
    Some(Role::Viewer) if role == Role::Viewer => Ok(()),
    _ => Err(AuthzError::Forbidden),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::error::Error as StdError;

  #[test]
  fn action_variants_eq() {
    assert_eq!(Action::Read, Action::Read);
    assert_ne!(Action::Read, Action::Create);
  }

  #[test]
  fn role_owner_admin_editor_viewer_custom() {
    assert_eq!(Role::Owner, Role::Owner);
    assert_eq!(Role::Custom("x".into()), Role::Custom("x".into()));
    assert_ne!(Role::Custom("a".into()), Role::Custom("b".into()));
  }

  #[test]
  fn authz_error_forbidden_display() {
    let e = AuthzError::Forbidden;
    let s = format!("{}", e);
    assert!(s.contains("Forbidden"));
    assert!(e.source().is_none());
  }

  #[test]
  fn authz_error_database_error_display_and_from() {
    let db_err = sea_orm::DbErr::Custom("connection failed".into());
    let e: AuthzError = db_err.into();
    let s = format!("{}", e);
    assert!(s.contains("Database error"));
    assert!(e.source().is_some());
  }

  #[derive(Debug, Clone)]
  struct MockContext {
    requester_id: Uuid,
    subject_id: Uuid,
    organization_id: Option<Uuid>,
    role: Option<Role>,
  }

  impl Default for MockContext {
    fn default() -> Self {
      let id = Uuid::new_v4();
      Self {
        requester_id: id,
        subject_id: id,
        organization_id: None,
        role: None,
      }
    }
  }

  impl AuthzContext for MockContext {
    type RequesterId = Uuid;
    type SubjectId = Uuid;

    fn requester_id(&self) -> Self::RequesterId {
      self.requester_id
    }
    fn subject_id(&self) -> Self::SubjectId {
      self.subject_id
    }
    fn organization_id(&self) -> Option<Uuid> {
      self.organization_id
    }
    fn role(&self) -> Option<Role> {
      self.role.clone()
    }
  }

  #[test]
  fn authz_context_mock_returns_ids_and_role() {
    let id = Uuid::new_v4();
    let org = Uuid::new_v4();
    let ctx = MockContext {
      requester_id: id,
      subject_id: id,
      organization_id: Some(org),
      role: Some(Role::Admin),
    };
    assert_eq!(ctx.requester_id(), id);
    assert_eq!(ctx.subject_id(), id);
    assert_eq!(ctx.organization_id(), Some(org));
    assert_eq!(ctx.role(), Some(Role::Admin));
  }

  struct AllowAllPolicy;

  #[async_trait::async_trait]
  impl ForgePolicy<()> for AllowAllPolicy {
    async fn can<C: AuthzContext>(
      &self,
      _context: &C,
      _action: Action,
      _resource: &(),
      _db: &impl sea_orm::ConnectionTrait,
    ) -> Result<bool, AuthzError> {
      Ok(true)
    }
  }

  #[tokio::test]
  async fn forge_policy_allow_all_returns_true() {
    let db = sea_orm::Database::connect("sqlite::memory:?mode=rwc")
      .await
      .unwrap();
    let policy = AllowAllPolicy;
    let ctx = MockContext::default();
    let ok = policy.can(&ctx, Action::Read, &(), &db).await.unwrap();
    assert!(ok);
  }
}
