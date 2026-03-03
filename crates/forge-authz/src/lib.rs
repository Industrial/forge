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

/// Blanket impl: exercised when AuthSession is obtained via axum_login's request flow (e.g. in forge-auth).
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

  // --- AuthzContext: default role() and custom impl ---
  #[derive(Debug, Clone)]
  struct MockContext {
    requester_id: uuid::Uuid,
    subject_id: uuid::Uuid,
    organization_id: Option<uuid::Uuid>,
    role: Option<Role>,
  }

  impl Default for MockContext {
    fn default() -> Self {
      let id = uuid::Uuid::new_v4();
      Self {
        requester_id: id,
        subject_id: id,
        organization_id: None,
        role: None,
      }
    }
  }

  impl AuthzContext for MockContext {
    fn requester_id(&self) -> uuid::Uuid {
      self.requester_id
    }
    fn subject_id(&self) -> uuid::Uuid {
      self.subject_id
    }
    fn organization_id(&self) -> Option<uuid::Uuid> {
      self.organization_id
    }
    fn role(&self) -> Option<Role> {
      self.role.clone()
    }
  }

  #[test]
  fn authz_context_default_role_is_none() {
    struct NoRoleContext(uuid::Uuid);
    impl AuthzContext for NoRoleContext {
      fn requester_id(&self) -> uuid::Uuid {
        self.0
      }
      fn subject_id(&self) -> uuid::Uuid {
        self.0
      }
      fn organization_id(&self) -> Option<uuid::Uuid> {
        None
      }
    }
    let ctx = NoRoleContext(uuid::Uuid::new_v4());
    assert!(ctx.role().is_none());
  }

  #[test]
  fn authz_context_mock_returns_ids_and_role() {
    let id = uuid::Uuid::new_v4();
    let org = uuid::Uuid::new_v4();
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

  // --- Action / Role clone and serde ---
  #[test]
  fn action_clone_and_serde_roundtrip() {
    for action in [
      Action::Read,
      Action::Create,
      Action::Update,
      Action::Delete,
      Action::Manage,
    ] {
      let cloned = action;
      assert_eq!(action, cloned);
      let json = serde_json::to_string(&action).unwrap();
      let back: Action = serde_json::from_str(&json).unwrap();
      assert_eq!(action, back);
    }
  }

  #[test]
  fn role_all_variants_and_serde_roundtrip() {
    assert_eq!(Role::Admin, Role::Admin);
    assert_eq!(Role::Editor, Role::Editor);
    assert_eq!(Role::Viewer, Role::Viewer);
    let custom = Role::Custom("role".to_string());
    let json = serde_json::to_string(&custom).unwrap();
    let back: Role = serde_json::from_str(&json).unwrap();
    assert_eq!(custom, back);
  }

  // --- ForgePolicy: mock policy and in-memory DB ---
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

  struct DenyAllPolicy;

  #[async_trait::async_trait]
  impl ForgePolicy<()> for DenyAllPolicy {
    async fn can<C: AuthzContext>(
      &self,
      _context: &C,
      _action: Action,
      _resource: &(),
      _db: &impl sea_orm::ConnectionTrait,
    ) -> Result<bool, AuthzError> {
      Ok(false)
    }
  }

  struct ErrPolicy;

  #[async_trait::async_trait]
  impl ForgePolicy<()> for ErrPolicy {
    async fn can<C: AuthzContext>(
      &self,
      _context: &C,
      _action: Action,
      _resource: &(),
      _db: &impl sea_orm::ConnectionTrait,
    ) -> Result<bool, AuthzError> {
      Err(AuthzError::Forbidden)
    }
  }

  #[tokio::test]
  async fn forge_policy_allow_all_returns_true() {
    let db = sea_orm::Database::connect("sqlite::memory:?mode=rwc")
      .await
      .unwrap();
    let policy = AllowAllPolicy;
    let ctx = MockContext::default();
    let ok = policy
      .can(&ctx, Action::Read, &(), &db)
      .await
      .unwrap();
    assert!(ok);
  }

  #[tokio::test]
  async fn forge_policy_deny_all_returns_false() {
    let db = sea_orm::Database::connect("sqlite::memory:?mode=rwc")
      .await
      .unwrap();
    let policy = DenyAllPolicy;
    let ctx = MockContext::default();
    let ok = policy
      .can(&ctx, Action::Delete, &(), &db)
      .await
      .unwrap();
    assert!(!ok);
  }

  #[tokio::test]
  async fn forge_policy_error_returns_err() {
    let db = sea_orm::Database::connect("sqlite::memory:?mode=rwc")
      .await
      .unwrap();
    let policy = ErrPolicy;
    let ctx = MockContext::default();
    let res = policy.can(&ctx, Action::Manage, &(), &db).await;
    assert!(matches!(res, Err(AuthzError::Forbidden)));
  }

  // --- Compile-time check: AuthSession<B> implements AuthzContext when B::User does ---
  #[test]
  fn auth_session_impl_authz_context_compile_check() {
    use axum_login::{AuthUser, AuthnBackend};

    #[derive(Debug, Clone)]
    struct MockAuthUser {
      id: uuid::Uuid,
    }
    impl AuthUser for MockAuthUser {
      type Id = uuid::Uuid;
      fn id(&self) -> Self::Id {
        self.id
      }
      fn session_auth_hash(&self) -> &[u8] {
        b"hash"
      }
    }
    impl AuthzContext for MockAuthUser {
      fn requester_id(&self) -> uuid::Uuid {
        self.id
      }
      fn subject_id(&self) -> uuid::Uuid {
        self.id
      }
      fn organization_id(&self) -> Option<uuid::Uuid> {
        None
      }
    }

    #[derive(Clone)]
    struct MockBackend;
    #[async_trait::async_trait]
    impl AuthnBackend for MockBackend {
      type User = MockAuthUser;
      type Credentials = ();
      type Error = std::convert::Infallible;
      async fn authenticate(
        &self,
        _creds: Self::Credentials,
      ) -> Result<Option<Self::User>, Self::Error> {
        Ok(None)
      }
      async fn get_user(
        &self,
        _user_id: &axum_login::UserId<Self>,
      ) -> Result<Option<Self::User>, Self::Error> {
        Ok(None)
      }
    }

    let user = MockAuthUser {
      id: uuid::Uuid::new_v4(),
    };
    assert_eq!(user.requester_id(), user.id);
    assert_eq!(user.subject_id(), user.id);
    assert!(user.organization_id().is_none());
    assert_eq!(user.id(), user.id);

    fn check_session_impl<B>()
    where
      B: AuthnBackend,
      B::User: AuthzContext,
    {
      fn require_authz_context<S: AuthzContext>() {}
      require_authz_context::<axum_login::AuthSession<B>>();
    }
    check_session_impl::<MockBackend>();
  }
}
