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
}

/// The authorization scope derived from request headers, e.g., `X-Organization-Id` and `X-Role-Id`.
/// Set by middleware; organization and role come from the DB so roles remain CRUD-able.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestScope {
  pub organization_id: Uuid,
  /// Role ID from `org_role` (CRUD-able).
  pub role_id: Uuid,
  /// Role name from `org_role.name`; used for permission lookups (e.g. `role_permission.role_name`).
  pub role_name: String,
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
  }

  impl Default for MockContext {
    fn default() -> Self {
      let id = Uuid::new_v4();
      Self {
        requester_id: id,
        subject_id: id,
        organization_id: None,
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
  }

  #[test]
  fn authz_context_mock_returns_ids_and_org() {
    let id = Uuid::new_v4();
    let org = Uuid::new_v4();
    let ctx = MockContext {
      requester_id: id,
      subject_id: id,
      organization_id: Some(org),
    };
    assert_eq!(ctx.requester_id(), id);
    assert_eq!(ctx.subject_id(), id);
    assert_eq!(ctx.organization_id(), Some(org));
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
