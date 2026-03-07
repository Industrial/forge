use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "api_tokens")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  pub user_id: Uuid,
  pub token_hash: String,
  pub name: Option<String>,
  pub last_used_at: Option<chrono::NaiveDateTime>,
  pub expires_at: Option<chrono::NaiveDateTime>,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod bdd_tests {
  use super::*;
  use chrono::Utc;
  use sea_orm::{Database, EntityTrait, Set};
  use sea_orm_migration::MigratorTrait;
  use uuid::Uuid;

  async fn test_db() -> forge_db::DbConnection {
    let conn = Database::connect(sea_orm::ConnectOptions::new("sqlite::memory:".to_string()))
      .await
      .unwrap();
    migrations::Migrator::up(&conn, None)
      .await
      .expect("migrate");
    forge_db::wrap_traced(conn)
  }

  async fn create_test_user(db: &forge_db::DbConnection, user_id: Uuid) {
    use crate::models::user;
    user::Entity::insert(user::ActiveModel {
      id: Set(user_id),
      email: Set(format!("test-{}@example.com", user_id)),
      password_hash: Set("test-hash".to_string()),
      is_active: Set(true),
      is_admin: Set(false),
      current_org_id: Set(None),
      current_role: Set(None),
      created_at: Set(Utc::now().naive_utc()),
      updated_at: Set(Utc::now().naive_utc()),
    })
    .exec(db)
    .await
    .expect("create test user");
  }

  mod model_structure_behavior {
    use super::*;

    #[tokio::test]
    async fn should_create_api_token_with_required_fields() {
      // Given: a test database with a user
      let db = test_db().await;
      let user_id = Uuid::new_v4();
      create_test_user(&db, user_id).await;
      let token_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();
      let token_hash = "hashed-token-value".to_string();

      // When: inserting an api_token with required fields
      let result = Entity::insert(ActiveModel {
        id: Set(token_id),
        user_id: Set(user_id),
        token_hash: Set(token_hash.clone()),
        name: Set(None),
        last_used_at: Set(None),
        expires_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await;

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_create_api_token_with_optional_fields() {
      // Given: a test database with a user
      let db = test_db().await;
      let user_id = Uuid::new_v4();
      create_test_user(&db, user_id).await;
      let token_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();
      let token_hash = "hashed-token-value".to_string();
      let name = Some("Test Token".to_string());
      let last_used = Some(now);
      let expires = Some(now + chrono::Duration::days(30));

      // When: inserting an api_token with all fields
      let result = Entity::insert(ActiveModel {
        id: Set(token_id),
        user_id: Set(user_id),
        token_hash: Set(token_hash.clone()),
        name: Set(name.clone()),
        last_used_at: Set(last_used),
        expires_at: Set(expires),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await;

      // Then: should succeed
      assert!(result.is_ok());

      // And: should be able to retrieve it
      let retrieved = Entity::find_by_id(token_id)
        .one(&db)
        .await
        .expect("find")
        .expect("token should exist");
      assert_eq!(retrieved.id, token_id);
      assert_eq!(retrieved.user_id, user_id);
      assert_eq!(retrieved.token_hash, token_hash);
      assert_eq!(retrieved.name, name);
    }

    #[tokio::test]
    async fn should_query_api_token_by_token_hash() {
      // Given: a test database with a user and an api_token
      let db = test_db().await;
      let user_id = Uuid::new_v4();
      create_test_user(&db, user_id).await;
      let token_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();
      let token_hash = "specific-hash-value".to_string();

      Entity::insert(ActiveModel {
        id: Set(token_id),
        user_id: Set(user_id),
        token_hash: Set(token_hash.clone()),
        name: Set(None),
        last_used_at: Set(None),
        expires_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert");

      // When: querying by token_hash
      let found = Entity::find()
        .filter(Column::TokenHash.eq(&token_hash))
        .one(&db)
        .await
        .expect("query");

      // Then: should find the token
      assert!(found.is_some());
      let token = found.unwrap();
      assert_eq!(token.id, token_id);
      assert_eq!(token.user_id, user_id);
    }

    #[tokio::test]
    async fn should_query_api_tokens_by_user_id() {
      // Given: a test database with multiple users and api_tokens
      let db = test_db().await;
      let user1_id = Uuid::new_v4();
      let user2_id = Uuid::new_v4();
      create_test_user(&db, user1_id).await;
      create_test_user(&db, user2_id).await;
      let now = Utc::now().naive_utc();

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user1_id),
        token_hash: Set("hash1".to_string()),
        name: Set(None),
        last_used_at: Set(None),
        expires_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert token1");

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user1_id),
        token_hash: Set("hash2".to_string()),
        name: Set(None),
        last_used_at: Set(None),
        expires_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert token2");

      Entity::insert(ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user2_id),
        token_hash: Set("hash3".to_string()),
        name: Set(None),
        last_used_at: Set(None),
        expires_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert token3");

      // When: querying tokens by user1_id
      let user1_tokens = Entity::find()
        .filter(Column::UserId.eq(user1_id))
        .all(&db)
        .await
        .expect("query");

      // Then: should find 2 tokens for user1
      assert_eq!(user1_tokens.len(), 2);
      assert!(user1_tokens.iter().all(|t| t.user_id == user1_id));
    }
  }

  mod expiration_behavior {
    use super::*;

    #[tokio::test]
    async fn should_store_expires_at_as_optional() {
      // Given: a test database with a user
      let db = test_db().await;
      let user_id = Uuid::new_v4();
      create_test_user(&db, user_id).await;
      let token_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();
      let future = now + chrono::Duration::days(30);

      // When: creating a token with expires_at
      Entity::insert(ActiveModel {
        id: Set(token_id),
        user_id: Set(user_id),
        token_hash: Set("hash".to_string()),
        name: Set(None),
        last_used_at: Set(None),
        expires_at: Set(Some(future)),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert");

      // Then: expires_at should be stored
      let token = Entity::find_by_id(token_id)
        .one(&db)
        .await
        .expect("find")
        .expect("token should exist");
      assert_eq!(token.expires_at, Some(future));
    }

    #[tokio::test]
    async fn should_allow_token_without_expiration() {
      // Given: a test database with a user
      let db = test_db().await;
      let user_id = Uuid::new_v4();
      create_test_user(&db, user_id).await;
      let token_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();

      // When: creating a token without expires_at
      Entity::insert(ActiveModel {
        id: Set(token_id),
        user_id: Set(user_id),
        token_hash: Set("hash".to_string()),
        name: Set(None),
        last_used_at: Set(None),
        expires_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert");

      // Then: expires_at should be None
      let token = Entity::find_by_id(token_id)
        .one(&db)
        .await
        .expect("find")
        .expect("token should exist");
      assert_eq!(token.expires_at, None);
    }
  }
}
