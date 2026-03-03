//! Authentication: password hashing, API token verification, and token/session extractors.

mod password;
mod token_auth;

pub use password::{hash_api_token, hash_password, verify_api_token, verify_password};
pub use token_auth::{OptionalRequireAuth, RequireAuth, TokenAuthLayer, TokenLookupFn, TokenUser};

/// Authentication backend trait.
///
/// This is a simplified version of `axum_login::AuthnBackend` for when the `session` feature is not enabled.
#[cfg(not(feature = "session"))]
pub trait Backend: Clone + Send + Sync + 'static {
    /// The user type.
    type User: AuthUser;
    /// The error type.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Attempts to authenticate a user given their credentials.
    fn authenticate(
        &self,
        token: String,
    ) -> impl std::future::Future<Output = Result<Option<Self::User>, Self::Error>> + Send;

    /// Gets a user by their ID.
    fn get_user(
        &self,
        user_id: <Self::User as AuthUser>::Id,
    ) -> impl std::future::Future<Output = Result<Option<Self::User>, Self::Error>> + Send;
}

/// A simplified version of `axum_login::AuthUser` for when the `session` feature is not enabled.
#[cfg(not(feature = "session"))]
pub trait AuthUser: Clone + Send + Sync + 'static {
    /// The user ID type.
    type Id: Clone + Send + Sync + 'static;

    /// Returns the user's ID.
    fn id(&self) -> Self::Id;
}

#[cfg(feature = "session")]
pub use axum_login::AuthnBackend as Backend;

#[cfg(not(feature = "session"))]
pub use self::Backend as AuthnBackend;

