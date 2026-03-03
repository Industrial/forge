//! Authentication and authorization for the Forge framework.
//!
//! - **password**: Argon2 password hashing and API token hashing (re-exported from [token_auth] for convenience).
//! - **token_auth**: Bearer token layer and extractors ([TokenUser], [RequireAuth], etc.).
//! - **authz**: Authorization context, roles, actions, and policy traits.

pub mod authz;
pub mod password;
pub mod token_auth;

pub use authz::{
  Action, AuthzContext, AuthzError, ForgePolicy, ForgeScoped, RequestScope, Role,
  TokenUserGuardExt, guard_user,
};
pub use axum_login::{AuthnBackend as Backend, AuthUser};
pub use token_auth::{
  OptionalRequireAuth, RequireAuth, TokenLookupFn, TokenUser,
  hash_api_token, hash_password, verify_api_token, verify_password,
};
