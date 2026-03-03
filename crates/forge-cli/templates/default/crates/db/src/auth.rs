use async_trait::async_trait;
use axum_login::AuthnBackend;
use forge_auth::token_auth::verify_password;
use forge_db::DbConnection;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;

use crate::models::user;

/// Auth backend error: database or password/validation errors.
#[derive(Debug)]
pub enum BackendError {
  Db(sea_orm::DbErr),
  Auth(forge_core::Error),
}

impl std::fmt::Display for BackendError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      BackendError::Db(e) => write!(f, "{}", e),
      BackendError::Auth(e) => write!(f, "{}", e),
    }
  }
}

impl std::error::Error for BackendError {}

impl From<sea_orm::DbErr> for BackendError {
  fn from(e: sea_orm::DbErr) -> Self {
    BackendError::Db(e)
  }
}

impl From<forge_core::Error> for BackendError {
  fn from(e: forge_core::Error) -> Self {
    BackendError::Auth(e)
  }
}

#[derive(Clone, Debug)]
pub struct Backend {
  db: DbConnection,
}

impl Backend {
  pub fn new(db: DbConnection) -> Self {
    Self { db }
  }
}

#[derive(Debug, Deserialize)]
pub struct Credentials {
  pub email: String,
  pub password: String,
}

#[async_trait]
impl AuthnBackend for Backend {
  type User = user::Model;
  type Credentials = Credentials;
  type Error = BackendError;

  async fn authenticate(
    &self,
    creds: Self::Credentials,
  ) -> Result<Option<Self::User>, Self::Error> {
    tracing::debug!(target: "app::auth::backend", "authenticate email={}", creds.email);
    let user = user::Entity::find()
      .filter(user::Column::Email.eq(creds.email))
      .one(&self.db)
      .await?;

    if let Some(user) = user {
      if verify_password(&creds.password, &user.password_hash)? {
        tracing::debug!(target: "app::auth::backend", "authenticate success user_id={}", user.id);
        return Ok(Some(user));
      }
    }

    tracing::debug!(target: "app::auth::backend", "authenticate failed (no user or bad password)");
    Ok(None)
  }

  async fn get_user(
    &self,
    user_id: &axum_login::UserId<Self>,
  ) -> Result<Option<Self::User>, BackendError> {
    tracing::debug!(target: "app::auth::backend", "get_user user_id={}", user_id);
    let user = user::Entity::find_by_id(*user_id).one(&self.db).await?;
    tracing::debug!(target: "app::auth::backend", "get_user result found={}", user.is_some());
    Ok(user)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use sea_orm::Database;

  #[tokio::test]
  async fn backend_new_creates_instance() {
    let conn = Database::connect(sea_orm::ConnectOptions::new(
      "sqlite::memory:".to_string(),
    ))
    .await
    .unwrap();
    let db = forge_db::wrap_traced(conn);
    let backend = Backend::new(db);
    let _ = backend;
  }

  #[test]
  fn credentials_deserialize() {
    let json = r#"{"email":"a@b.com","password":"secret"}"#;
    let c: Credentials = serde_json::from_str(json).unwrap();
    assert_eq!(c.email, "a@b.com");
    assert_eq!(c.password, "secret");
  }
}
