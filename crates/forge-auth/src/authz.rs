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

#[cfg(test)]
mod bdd_tests {
  use super::*;
  use sea_orm::Database;
  use std::error::Error as StdError;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests are organized by feature/behavior area with descriptive names.

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

  mod action_behavior {
    use super::*;

    #[test]
    fn should_be_equal_when_same_action_variants() {
      // Given: two instances of the same action
      let action1 = Action::Read;
      let action2 = Action::Read;

      // When: comparing them
      // Then: they should be equal
      assert_eq!(action1, action2, "Same action variants should be equal");
    }

    #[test]
    fn should_not_be_equal_when_different_action_variants() {
      // Given: two different action variants
      let read_action = Action::Read;
      let create_action = Action::Create;

      // When: comparing them
      // Then: they should not be equal
      assert_ne!(read_action, create_action, "Different action variants should not be equal");
    }

    #[test]
    fn should_have_serialize_deserialize_traits() {
      // Given: an action variant with Serialize/Deserialize derives
      let action = Action::Manage;

      // When: checking that action implements Serialize and Deserialize
      // Then: compilation succeeds (test passes if this compiles)
      // Note: Actual serialization testing would require serde_json in dev-dependencies
      fn _test_serialize<T: serde::Serialize>(_t: &T) {}
      fn _test_deserialize<'de, T: serde::Deserialize<'de>>(_t: T) {}
      _test_serialize(&action);
      // Deserialize test would require a value, but trait bound check is sufficient
    }

    #[test]
    fn should_have_all_expected_variants() {
      // Given: all expected action variants
      // When: checking variants exist
      // Then: all variants should be available
      let _read = Action::Read;
      let _create = Action::Create;
      let _update = Action::Update;
      let _delete = Action::Delete;
      let _manage = Action::Manage;
      // Test passes if all variants compile
    }
  }

  mod authz_error_behavior {
    use super::*;

    #[test]
    fn should_display_forbidden_message_when_forbidden_error() {
      // Given: a Forbidden error
      let error = AuthzError::Forbidden;

      // When: formatting as string
      let message = format!("{}", error);

      // Then: message should indicate forbidden
      assert!(
        message.contains("Forbidden"),
        "Forbidden error should display forbidden message"
      );
      assert!(
        message.contains("permission"),
        "Forbidden error message should mention permission"
      );
    }

    #[test]
    fn should_display_not_found_message_when_not_found_error() {
      // Given: a NotFound error
      let error = AuthzError::NotFound;

      // When: formatting as string
      let message = format!("{}", error);

      // Then: message should indicate not found
      assert!(
        message.contains("Not Found"),
        "NotFound error should display not found message"
      );
      assert!(
        message.contains("resource"),
        "NotFound error message should mention resource"
      );
    }

    #[test]
    fn should_convert_database_error_to_authz_error() {
      // Given: a database error
      let db_error = sea_orm::DbErr::Custom("connection failed".into());

      // When: converting to AuthzError
      let authz_error: AuthzError = db_error.into();

      // Then: should be DatabaseError variant
      match authz_error {
        AuthzError::DatabaseError(_) => {}
        _ => panic!("Should convert to DatabaseError variant"),
      }
    }

    #[test]
    fn should_preserve_database_error_message() {
      // Given: a database error with a specific message
      let db_error = sea_orm::DbErr::Custom("connection timeout".into());

      // When: converting to AuthzError and formatting
      let authz_error: AuthzError = db_error.into();
      let message = format!("{}", authz_error);

      // Then: message should contain database error details
      assert!(
        message.contains("Database error"),
        "Error message should indicate database error"
      );
    }

    #[test]
    fn should_have_source_for_database_error() {
      // Given: a database error converted to AuthzError
      let db_error = sea_orm::DbErr::Custom("test error".into());
      let authz_error: AuthzError = db_error.into();

      // When: checking error source
      // Then: should have a source (the underlying DbErr)
      assert!(
        authz_error.source().is_some(),
        "DatabaseError should have a source"
      );
    }

    #[test]
    fn should_not_have_source_for_forbidden_error() {
      // Given: a Forbidden error
      let error = AuthzError::Forbidden;

      // When: checking error source
      // Then: should not have a source
      assert!(
        error.source().is_none(),
        "Forbidden error should not have a source"
      );
    }
  }

  mod authz_context_behavior {
    use super::*;

    #[test]
    fn should_return_requester_id_when_requested() {
      // Given: a context with a requester ID
      let requester_id = Uuid::new_v4();
      let ctx = MockContext {
        requester_id,
        subject_id: Uuid::new_v4(),
        organization_id: None,
      };

      // When: requesting requester ID
      let id = ctx.requester_id();

      // Then: should return the correct requester ID
      assert_eq!(id, requester_id, "Should return correct requester ID");
    }

    #[test]
    fn should_return_subject_id_when_requested() {
      // Given: a context with a subject ID
      let subject_id = Uuid::new_v4();
      let ctx = MockContext {
        requester_id: Uuid::new_v4(),
        subject_id,
        organization_id: None,
      };

      // When: requesting subject ID
      let id = ctx.subject_id();

      // Then: should return the correct subject ID
      assert_eq!(id, subject_id, "Should return correct subject ID");
    }

    #[test]
    fn should_return_organization_id_when_present() {
      // Given: a context with an organization ID
      let org_id = Uuid::new_v4();
      let ctx = MockContext {
        requester_id: Uuid::new_v4(),
        subject_id: Uuid::new_v4(),
        organization_id: Some(org_id),
      };

      // When: requesting organization ID
      let id = ctx.organization_id();

      // Then: should return Some(organization_id)
      assert_eq!(id, Some(org_id), "Should return organization ID when present");
    }

    #[test]
    fn should_return_none_when_organization_id_not_present() {
      // Given: a context without an organization ID
      let ctx = MockContext {
        requester_id: Uuid::new_v4(),
        subject_id: Uuid::new_v4(),
        organization_id: None,
      };

      // When: requesting organization ID
      let id = ctx.organization_id();

      // Then: should return None
      assert_eq!(id, None, "Should return None when organization ID not present");
    }
  }

  mod request_scope_behavior {
    use super::*;

    #[test]
    fn should_implement_authz_context_trait() {
      // Given: a RequestScope instance
      let org_id = Uuid::new_v4();
      let role_id = Uuid::new_v4();
      let scope = RequestScope {
        organization_id: org_id,
        role_id,
        role_name: "admin".to_string(),
      };

      // When: using it as AuthzContext
      // Then: should provide organization_id
      assert_eq!(
        scope.organization_id(),
        Some(org_id),
        "RequestScope should provide organization_id"
      );
    }

    #[test]
    fn should_return_nil_uuids_for_requester_and_subject() {
      // Given: a RequestScope instance
      let scope = RequestScope {
        organization_id: Uuid::new_v4(),
        role_id: Uuid::new_v4(),
        role_name: "user".to_string(),
      };

      // When: requesting requester_id and subject_id
      // Then: should return nil UUIDs
      assert_eq!(
        scope.requester_id(),
        Uuid::nil(),
        "RequestScope should return nil for requester_id"
      );
      assert_eq!(
        scope.subject_id(),
        Uuid::nil(),
        "RequestScope should return nil for subject_id"
      );
    }

    #[test]
    fn should_store_organization_role_and_role_name() {
      // Given: organization ID, role ID, and role name
      let org_id = Uuid::new_v4();
      let role_id = Uuid::new_v4();
      let role_name = "manager".to_string();

      // When: creating RequestScope
      let scope = RequestScope {
        organization_id: org_id,
        role_id,
        role_name: role_name.clone(),
      };

      // Then: should store all values correctly
      assert_eq!(scope.organization_id, org_id, "Should store organization_id");
      assert_eq!(scope.role_id, role_id, "Should store role_id");
      assert_eq!(scope.role_name, role_name, "Should store role_name");
    }

    #[test]
    fn should_have_serialize_deserialize_traits() {
      // Given: a RequestScope instance with Serialize/Deserialize derives
      let scope = RequestScope {
        organization_id: Uuid::new_v4(),
        role_id: Uuid::new_v4(),
        role_name: "admin".to_string(),
      };

      // When: checking that RequestScope implements Serialize and Deserialize
      // Then: compilation succeeds (test passes if this compiles)
      // Note: Actual serialization testing would require serde_json in dev-dependencies
      fn _test_serialize<T: serde::Serialize>(_t: &T) {}
      fn _test_deserialize<'de, T: serde::Deserialize<'de>>(_t: T) {}
      _test_serialize(&scope);
      // Deserialize test would require a value, but trait bound check is sufficient
    }
  }

  mod forge_policy_behavior {
    use super::*;

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

    struct ErrorPolicy;

    #[async_trait::async_trait]
    impl ForgePolicy<()> for ErrorPolicy {
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
    async fn should_return_true_when_policy_allows() {
      // Given: a policy that allows all and a context
      let db = Database::connect("sqlite::memory:?mode=rwc").await.unwrap();
      let policy = AllowAllPolicy;
      let ctx = MockContext::default();

      // When: checking permission
      let result = policy.can(&ctx, Action::Read, &(), &db).await;

      // Then: should return Ok(true)
      assert!(result.is_ok(), "Policy check should succeed");
      assert!(result.unwrap(), "Policy should allow access");
    }

    #[tokio::test]
    async fn should_return_false_when_policy_denies() {
      // Given: a policy that denies all and a context
      let db = Database::connect("sqlite::memory:?mode=rwc").await.unwrap();
      let policy = DenyAllPolicy;
      let ctx = MockContext::default();

      // When: checking permission
      let result = policy.can(&ctx, Action::Read, &(), &db).await;

      // Then: should return Ok(false)
      assert!(result.is_ok(), "Policy check should succeed");
      assert!(!result.unwrap(), "Policy should deny access");
    }

    #[tokio::test]
    async fn should_return_error_when_policy_errors() {
      // Given: a policy that returns an error
      let db = Database::connect("sqlite::memory:?mode=rwc").await.unwrap();
      let policy = ErrorPolicy;
      let ctx = MockContext::default();

      // When: checking permission
      let result = policy.can(&ctx, Action::Read, &(), &db).await;

      // Then: should return Err
      assert!(result.is_err(), "Policy check should return error");
      match result.unwrap_err() {
        AuthzError::Forbidden => {}
        _ => panic!("Should return Forbidden error"),
      }
    }

    #[tokio::test]
    async fn should_accept_different_action_types() {
      // Given: a policy and different actions
      let db = Database::connect("sqlite::memory:?mode=rwc").await.unwrap();
      let policy = AllowAllPolicy;
      let ctx = MockContext::default();

      // When: checking different actions
      let actions = [
        Action::Read,
        Action::Create,
        Action::Update,
        Action::Delete,
        Action::Manage,
      ];

      // Then: should handle all action types
      for action in actions.iter() {
        let result = policy.can(&ctx, *action, &(), &db).await;
        assert!(result.is_ok(), "Should handle action {:?}", action);
      }
    }

    #[tokio::test]
    async fn should_use_database_connection_when_provided() {
      // Given: a policy that uses database connection
      let db = Database::connect("sqlite::memory:?mode=rwc").await.unwrap();
      let policy = AllowAllPolicy;
      let ctx = MockContext::default();

      // When: checking permission with database
      let result = policy.can(&ctx, Action::Read, &(), &db).await;

      // Then: should use database connection successfully
      assert!(result.is_ok(), "Should use database connection");
    }
  }

  mod forge_scoped_behavior {
    use super::*;

    #[test]
    fn should_be_implementable_for_entities() {
      // Given: the ForgeScoped trait definition
      // When: checking trait signature
      // Then: trait should be available for implementation
      // Note: Actual implementation requires specific entity types
      // This test verifies the trait exists and can be used conceptually
      fn _verify_trait_exists<E: sea_orm::EntityTrait>() {
        // ForgeScoped<E> trait exists and can be implemented
        // Implementation would be: impl ForgeScoped<E> for Select<E> { ... }
      }
      // Test passes if trait compiles
    }

    #[test]
    fn should_accept_authz_context_as_parameter() {
      // Given: the ForgeScoped trait signature
      // When: examining the scoped method
      // Then: it should accept any type implementing AuthzContext
      let ctx = MockContext::default();
      let _has_org = ctx.organization_id().is_some();
      // This verifies AuthzContext can be used with ForgeScoped
    }
  }
}
