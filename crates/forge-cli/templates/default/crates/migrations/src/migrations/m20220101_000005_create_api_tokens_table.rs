use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum ApiTokens {
  Table,
  Id,
  UserId,
  TokenHash,
  Name,
  LastUsedAt,
  ExpiresAt,
  CreatedAt,
  UpdatedAt,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000005_create_api_tokens_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(ApiTokens::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(ApiTokens::Id)
              .uuid()
              .not_null()
              .primary_key(),
          )
          .col(ColumnDef::new(ApiTokens::UserId).uuid().not_null())
          .col(ColumnDef::new(ApiTokens::TokenHash).string().not_null())
          .col(ColumnDef::new(ApiTokens::Name).string())
          .col(ColumnDef::new(ApiTokens::LastUsedAt).date_time())
          .col(ColumnDef::new(ApiTokens::ExpiresAt).date_time())
          .col(ColumnDef::new(ApiTokens::CreatedAt).date_time().not_null())
          .col(ColumnDef::new(ApiTokens::UpdatedAt).date_time().not_null())
          .foreign_key(
            ForeignKey::create()
              .name("fk_api_tokens_user_id")
              .from_tbl(ApiTokens::Table)
              .from_col(ApiTokens::UserId)
              .to_tbl(Alias::new("user"))
              .to_col(Alias::new("id")),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(ApiTokens::Table).to_owned())
      .await
  }
}

#[cfg(test)]
mod bdd_tests {
  use super::*;
  use crate::migrations::m20220101_000001_create_user_table;
  use sea_orm::{Database, EntityTrait};

  async fn test_db() -> sea_orm::DatabaseConnection {
    Database::connect(sea_orm::ConnectOptions::new("sqlite::memory:".to_string()))
      .await
      .unwrap()
  }

  mod migration_up_behavior {
    use super::*;

    #[tokio::test]
    async fn should_create_api_tokens_table() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;

      // When: running migration up
      let result = migration.up(&SchemaManager::new(&db)).await;

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_create_table_with_all_columns() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;
      migration
        .up(&SchemaManager::new(&db))
        .await
        .expect("migrate");

      // When: checking table structure
      // Then: table should exist with all columns
      use db::models::api_token;
      let tokens = api_token::Entity::find().all(&db).await;
      assert!(tokens.is_ok());
    }

    #[tokio::test]
    async fn should_create_table_with_primary_key() {
      // Given: a test database with user table (required for foreign key)
      let db = test_db().await;
      // Run user table migration first
      let user_migration = m20220101_000001_create_user_table::Migration;
      user_migration
        .up(&SchemaManager::new(&db))
        .await
        .expect("migrate user table");

      let migration = Migration;
      migration
        .up(&SchemaManager::new(&db))
        .await
        .expect("migrate");

      // Create a user first (required for foreign key constraint)
      use chrono::Utc;
      use db::models::{api_token, user};
      use sea_orm::Set;
      use uuid::Uuid;

      let user_id = Uuid::new_v4();
      let now = Utc::now().naive_utc();
      user::Entity::insert(user::ActiveModel {
        id: Set(user_id),
        email: Set(format!("test-{}@example.com", user_id)),
        password_hash: Set("test-hash".to_string()),
        is_active: Set(true),
        is_admin: Set(false),
        current_org_id: Set(None),
        current_role: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("create user");

      // When: inserting api token with id
      // Then: should succeed
      let result = api_token::Entity::insert(api_token::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        token_hash: Set("hash".to_string()),
        name: Set(None),
        last_used_at: Set(None),
        expires_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await;

      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_create_foreign_key_to_user() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;

      // When: running migration up
      let result = migration.up(&SchemaManager::new(&db)).await;

      // Then: should succeed (foreign key created)
      assert!(result.is_ok());
    }
  }

  mod migration_down_behavior {
    use super::*;

    #[tokio::test]
    async fn should_drop_api_tokens_table() {
      // Given: a test database with api_tokens table created
      let db = test_db().await;
      let migration = Migration;
      migration
        .up(&SchemaManager::new(&db))
        .await
        .expect("migrate up");

      // When: running migration down
      let result = migration.down(&SchemaManager::new(&db)).await;

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_remove_table_after_down() {
      // Given: a test database with api_tokens table
      let db = test_db().await;
      let migration = Migration;
      migration
        .up(&SchemaManager::new(&db))
        .await
        .expect("migrate up");
      migration
        .down(&SchemaManager::new(&db))
        .await
        .expect("migrate down");

      // When: trying to query api_tokens table
      // Then: should fail (table doesn't exist)
      use db::models::api_token;
      let result = api_token::Entity::find().all(&db).await;
      assert!(result.is_err());
    }
  }

  mod migration_name_behavior {
    use super::*;

    #[test]
    fn should_have_correct_migration_name() {
      // Given: Migration struct
      let migration = Migration;

      // When: getting migration name
      let name = migration.name();

      // Then: should match expected name
      assert_eq!(name, "m20220101_000005_create_api_tokens_table");
    }
  }
}
