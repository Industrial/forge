//! Forge Authorization Engine
//!
//! Implements a "Shallow Gate + Deep Scope" authorization strategy for high-performance
//! SaaS applications using SeaORM and Axum.

use async_trait::async_trait;
use sea_orm::{EntityTrait, Select};
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
/// Distinguishes between who is holding the session and who they are acting as.
pub trait AuthzContext {
  /// The ID of the actual session holder.
  fn requester_id(&self) -> uuid::Uuid;

  /// The ID of the identity context (Subject).
  fn subject_id(&self) -> uuid::Uuid;

  /// Optional organization/tenant context.
  fn organization_id(&self) -> Option<uuid::Uuid>;

  /// Optional role in the current organization (for Shallow Gate).
  /// Default is None; implement to enable role-based guards.
  fn role(&self) -> Option<Role> {
    None
  }
}

impl<B> AuthzContext for axum_login::AuthSession<B>
where
  B: axum_login::AuthnBackend,
  B::User: AuthzContext,
{
  fn requester_id(&self) -> uuid::Uuid {
    self
      .user
      .as_ref()
      .map(|u| u.requester_id())
      .unwrap_or_else(uuid::Uuid::nil)
  }

  fn subject_id(&self) -> uuid::Uuid {
    self
      .user
      .as_ref()
      .map(|u| u.subject_id())
      .unwrap_or_else(uuid::Uuid::nil)
  }

  fn organization_id(&self) -> Option<uuid::Uuid> {
    self.user.as_ref().and_then(|u| u.organization_id())
  }

  fn role(&self) -> Option<Role> {
    self.user.as_ref().and_then(|u| u.role())
  }
}

/// Trait for implicitly scoping SeaORM queries.
/// This is the "Deep Scope" primitive.
pub trait ForgeScoped<E: EntityTrait> {
  /// Apply authorization filters to a SeaORM query.
  fn scoped<C: AuthzContext>(self, context: &C) -> Select<E>;
}

/// Trait for explicit ReBAC policy logic.
/// This is the "Logic" primitive.
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

// Macros will provide specific implementations for Entities that derive ForgeScoped.

#[cfg(test)]
mod tests {
  use super::*;
  use std::error::Error as StdError;

  #[test]
  fn action_variants_eq() {
    assert_eq!(Action::Read, Action::Read);
    assert_ne!(Action::Read, Action::Create);
    assert_eq!(Action::Create, Action::Create);
    assert_eq!(Action::Update, Action::Update);
    assert_eq!(Action::Delete, Action::Delete);
    assert_eq!(Action::Manage, Action::Manage);
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
  fn authz_error_not_found_display() {
    let e = AuthzError::NotFound;
    let s = format!("{}", e);
    assert!(s.contains("Not Found"));
  }

  #[test]
  fn authz_error_database_error_display_and_from() {
    let db_err = sea_orm::DbErr::Custom("connection failed".into());
    let e: AuthzError = db_err.into();
    let s = format!("{}", e);
    assert!(s.contains("Database error"));
    assert!(e.source().is_some());
  }
}
