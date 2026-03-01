use async_trait::async_trait;
use forge::auth::verify_password;
use forge::{axum_login::AuthnBackend, DbConnection, Error};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;

use crate::models::user;

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
  type Error = Error;

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

  async fn get_user(&self, user_id: &forge::axum_login::UserId<Self>) -> Result<Option<Self::User>, Error> {
    tracing::debug!(target: "app::auth::backend", "get_user user_id={}", user_id);
    let user = user::Entity::find_by_id(*user_id)
      .one(&self.db)
      .await?;
    tracing::debug!(target: "app::auth::backend", "get_user result found={}", user.is_some());
    Ok(user)
  }
}
