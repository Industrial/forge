//! Authentication and authorization for the Forge framework.
//!
//! - **password**: Argon2 password hashing and API token hashing (re-exported from [token_auth] for convenience).
//! - **token_auth**: Bearer token layer and extractors ([TokenUser], [RequireAuth], etc.).
//! - **authz**: Authorization context, roles, actions, and policy traits.

pub mod authz;
pub mod password;
pub mod token_auth;

pub use authz::{Action, AuthzContext, AuthzError, ForgePolicy, ForgeScoped, RequestScope};
pub use axum_login::{AuthUser, AuthnBackend as Backend};
pub use token_auth::{
  OptionalRequireAuth, RequireAuth, TokenLookupFn, TokenUser, hash_api_token, hash_password,
  verify_api_token, verify_password,
};

#[cfg(test)]
mod bdd_tests {
  use super::*;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests verify that all public API re-exports are accessible and functional.
  mod public_api_exports {
    use super::*;

    #[test]
    fn should_export_authz_types() {
      // Given: lib.rs re-exports authz types
      // When: I reference the re-exported types
      let _action: Action = Action::Read;
      let _error: AuthzError = AuthzError::Forbidden;
      let _scope: RequestScope = RequestScope {
        organization_id: uuid::Uuid::new_v4(),
        role_id: uuid::Uuid::new_v4(),
        role_name: "test_role".to_string(),
      };

      // Then: all types should be accessible and usable
      // (compilation success means types are exported correctly)
    }

    #[test]
    fn should_export_authz_traits() {
      // Given: lib.rs re-exports authz traits
      // When: I reference the re-exported traits
      use sea_orm::EntityTrait;
      fn _test_forge_policy<R, P: ForgePolicy<R>>() {}
      fn _test_forge_scoped<E: EntityTrait, S: ForgeScoped<E>>() {}
      fn _test_authz_context<C: AuthzContext>() {}

      // Then: all traits should be accessible and usable
      // (compilation success means traits are exported correctly)
    }

    #[test]
    fn should_export_axum_login_types() {
      // Given: lib.rs re-exports axum_login types
      // When: I reference the re-exported types
      fn _test_auth_user<U: AuthUser>() {}
      fn _test_backend<B: Backend>() {}

      // Then: all types should be accessible and usable
      // (compilation success means types are exported correctly)
    }

    #[test]
    fn should_export_token_auth_types() {
      // Given: lib.rs re-exports token_auth types
      // When: I reference the re-exported types
      fn _test_token_lookup_fn(_fn: TokenLookupFn) {}
      fn _test_token_user<U: AuthzContext>(_user: TokenUser<U>) {}
      // RequireAuth and OptionalRequireAuth are structs with trait bounds
      // We verify they're exported by checking they can be used in type positions
      fn _test_require_auth<B: Backend<User = U>, U: AuthUser>(_auth: RequireAuth<B, U>) {}
      fn _test_optional_require_auth<B: Backend<User = U>, U: AuthUser>(
        _auth: OptionalRequireAuth<B, U>,
      ) {
      }

      // Then: all types should be accessible and usable
      // (compilation success means types are exported correctly)
    }

    #[test]
    fn should_export_password_functions() {
      // Given: lib.rs re-exports password hashing functions
      // When: I reference the re-exported functions
      let _hash_password_fn: fn(&str) -> Result<String, forge_core::Error> = hash_password;
      let _verify_password_fn: fn(&str, &str) -> Result<bool, forge_core::Error> = verify_password;

      // Then: all functions should be accessible and callable
      // (compilation success means functions are exported correctly)
    }

    #[test]
    fn should_export_api_token_functions() {
      // Given: lib.rs re-exports API token hashing functions
      // When: I reference the re-exported functions
      let _hash_api_token_fn: fn(&str) -> String = hash_api_token;
      let _verify_api_token_fn: fn(&str, &str) -> bool = verify_api_token;

      // Then: all functions should be accessible and callable
      // (compilation success means functions are exported correctly)
    }
  }

  mod re_export_integration {
    use super::*;

    #[test]
    fn should_allow_using_authz_types_from_lib() {
      // Given: I import Action from lib.rs
      // When: I use the Action enum
      let read_action = Action::Read;
      let create_action = Action::Create;
      let update_action = Action::Update;
      let delete_action = Action::Delete;
      let manage_action = Action::Manage;

      // Then: all variants should be accessible and usable
      assert_eq!(read_action, Action::Read);
      assert_eq!(create_action, Action::Create);
      assert_eq!(update_action, Action::Update);
      assert_eq!(delete_action, Action::Delete);
      assert_eq!(manage_action, Action::Manage);
    }

    #[test]
    fn should_allow_using_password_functions_from_lib() {
      // Given: I import password functions from lib.rs
      // When: I call hash_password with a password
      let password = "test_password_123";
      let hashed = hash_password(password).expect("hash should succeed");

      // Then: it should return a hashed password string
      assert!(!hashed.is_empty());
      assert_ne!(hashed, password);
      assert!(hashed.starts_with("$argon2"));

      // When: I verify the password against the hash
      let is_valid = verify_password(password, &hashed).expect("verify should succeed");

      // Then: verification should succeed
      assert!(is_valid);
    }

    #[test]
    fn should_allow_using_api_token_functions_from_lib() {
      // Given: I import API token functions from lib.rs
      // When: I call hash_api_token with a token
      let token = "test_api_token_456";
      let hashed = hash_api_token(token);

      // Then: it should return a hashed token string (SHA-256 hex, not Argon2)
      assert!(!hashed.is_empty());
      assert_ne!(hashed, token);
      assert_eq!(hashed.len(), 64, "SHA-256 hex digest is 64 chars");

      // When: I verify the token against the hash
      let is_valid = verify_api_token(token, &hashed);

      // Then: verification should succeed
      assert!(is_valid);
    }

    #[test]
    fn should_reject_invalid_password_verification() {
      // Given: I have a hashed password
      let password = "correct_password";
      let hashed = hash_password(password).expect("hash should succeed");

      // When: I verify with an incorrect password
      let is_valid = verify_password("wrong_password", &hashed).expect("verify should succeed");

      // Then: verification should fail
      assert!(!is_valid);
    }

    #[test]
    fn should_reject_invalid_api_token_verification() {
      // Given: I have a hashed API token
      let token = "correct_token";
      let hashed = hash_api_token(token);

      // When: I verify with an incorrect token
      let is_valid = verify_api_token("wrong_token", &hashed);

      // Then: verification should fail
      assert!(!is_valid);
    }

    #[test]
    fn should_export_authz_error_variants() {
      // Given: I import AuthzError from lib.rs
      // When: I create error instances
      let forbidden = AuthzError::Forbidden;
      let not_found = AuthzError::NotFound;

      // Then: all error variants should be accessible
      assert!(matches!(forbidden, AuthzError::Forbidden));
      assert!(matches!(not_found, AuthzError::NotFound));

      // And: error messages should be descriptive
      assert!(forbidden.to_string().contains("Forbidden"));
      assert!(not_found.to_string().contains("Not Found"));
    }

    #[test]
    fn should_create_request_scope_instances() {
      // Given: I import RequestScope from lib.rs
      // When: I create scope instances
      let scope = RequestScope {
        organization_id: uuid::Uuid::new_v4(),
        role_id: uuid::Uuid::new_v4(),
        role_name: "admin".to_string(),
      };

      // Then: scope should be accessible and usable
      assert_eq!(scope.role_name, "admin");
      assert!(scope.organization_id != uuid::Uuid::nil());
      assert!(scope.role_id != uuid::Uuid::nil());
    }
  }
}
