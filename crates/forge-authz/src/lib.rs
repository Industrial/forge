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
    db: &sea_orm::DatabaseConnection,
  ) -> Result<bool, AuthzError>;
}

// Macros will provide specific implementations for Entities that derive ForgeScoped.
