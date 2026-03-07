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

    if let Some(user) = user
      && verify_password(&creds.password, &user.password_hash)?
    {
      tracing::debug!(target: "app::auth::backend", "authenticate success user_id={}", user.id);
      return Ok(Some(user));
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
  use sea_orm::{Database, Set};

  #[tokio::test]
  async fn backend_new_creates_instance() {
    let conn = Database::connect(sea_orm::ConnectOptions::new("sqlite::memory:".to_string()))
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

  mod bdd_tests {
    use super::*;
    use crate::models::user;
    use chrono::Utc;
    use forge_auth::token_auth::hash_password;
    use sea_orm_migration::MigratorTrait;
    use uuid::Uuid;

    async fn test_db_with_user(email: &str, password: &str) -> (Backend, Uuid) {
      let conn = Database::connect(sea_orm::ConnectOptions::new("sqlite::memory:".to_string()))
        .await
        .unwrap();
      migrations::Migrator::up(&conn, None)
        .await
        .expect("migrate");
      let db = forge_db::wrap_traced(conn);
      let now = Utc::now().naive_utc();
      let user_id = Uuid::new_v4();
      let password_hash = hash_password(password).expect("hash");
      let model = user::ActiveModel {
        id: Set(user_id),
        email: Set(email.to_string()),
        password_hash: Set(password_hash),
        is_active: Set(true),
        is_admin: Set(false),
        current_org_id: Set(None),
        current_role: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      };
      user::Entity::insert(model)
        .exec(&db)
        .await
        .expect("insert user");
      (Backend::new(db), user_id)
    }

    mod authenticate_behavior {
      use super::*;

      #[tokio::test]
      async fn should_return_some_user_when_email_and_password_match() {
        // Given: a backend with a user "u@test.com" and password "secret"
        let (backend, _) = test_db_with_user("u@test.com", "secret").await;
        // When: authenticating with those credentials
        let creds = Credentials {
          email: "u@test.com".to_string(),
          password: "secret".to_string(),
        };
        let result = backend.authenticate(creds).await.unwrap();
        // Then: should return Some(user) with matching email
        let user = result.expect("some user");
        assert_eq!(user.email, "u@test.com");
      }

      #[tokio::test]
      async fn should_return_none_when_password_is_wrong() {
        // Given: a backend with a user "u@test.com" and password "secret"
        let (backend, _) = test_db_with_user("u@test.com", "secret").await;
        // When: authenticating with correct email but wrong password
        let creds = Credentials {
          email: "u@test.com".to_string(),
          password: "wrong".to_string(),
        };
        let result = backend.authenticate(creds).await.unwrap();
        // Then: should return None
        assert!(result.is_none());
      }

      #[tokio::test]
      async fn should_return_none_when_email_does_not_exist() {
        // Given: a backend with a user "u@test.com"
        let (backend, _) = test_db_with_user("u@test.com", "secret").await;
        // When: authenticating with an unknown email
        let creds = Credentials {
          email: "other@test.com".to_string(),
          password: "secret".to_string(),
        };
        let result = backend.authenticate(creds).await.unwrap();
        // Then: should return None
        assert!(result.is_none());
      }
    }

    mod get_user_behavior {
      use super::*;

      #[tokio::test]
      async fn should_return_some_user_when_id_exists() {
        // Given: a backend with a user
        let (backend, user_id) = test_db_with_user("u@test.com", "secret").await;
        // When: getting user by that id
        let uid = axum_login::UserId::<Backend>::from(user_id);
        let result = backend.get_user(&uid).await.unwrap();
        // Then: should return Some(user) with matching id
        let user = result.expect("some user");
        assert_eq!(user.id, user_id);
        assert_eq!(user.email, "u@test.com");
      }

      #[tokio::test]
      async fn should_return_none_when_id_does_not_exist() {
        // Given: a backend with a user
        let (backend, _) = test_db_with_user("u@test.com", "secret").await;
        // When: getting user by a random id
        let random_id = Uuid::new_v4();
        let uid = axum_login::UserId::<Backend>::from(random_id);
        let result = backend.get_user(&uid).await.unwrap();
        // Then: should return None
        assert!(result.is_none());
      }
    }
  }
}
