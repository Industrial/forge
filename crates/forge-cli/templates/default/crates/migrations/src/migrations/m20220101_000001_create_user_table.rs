use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum User {
  Table,
  Id,
  Email,
  PasswordHash,
  IsActive,
  IsAdmin,
  CurrentOrgId,
  CurrentRole,
  CreatedAt,
  UpdatedAt,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000001_create_user_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(User::Table)
          .if_not_exists()
          .col(ColumnDef::new(User::Id).uuid().not_null().primary_key())
          .col(ColumnDef::new(User::Email).string().unique_key().not_null())
          .col(ColumnDef::new(User::PasswordHash).string().not_null())
          .col(
            ColumnDef::new(User::IsActive)
              .boolean()
              .not_null()
              .default(true),
          )
          .col(
            ColumnDef::new(User::IsAdmin)
              .boolean()
              .not_null()
              .default(false),
          )
          .col(ColumnDef::new(User::CurrentOrgId).uuid())
          .col(ColumnDef::new(User::CurrentRole).string())
          .col(ColumnDef::new(User::CreatedAt).date_time().not_null())
          .col(ColumnDef::new(User::UpdatedAt).date_time().not_null())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(User::Table).to_owned())
      .await
  }
}

#[cfg(test)]
mod bdd_tests {
  use super::*;
  use sea_orm::{ConnectionTrait, Database, EntityTrait};
  use sea_orm_migration::prelude::*;

  async fn test_db() -> sea_orm::DatabaseConnection {
    Database::connect(sea_orm::ConnectOptions::new("sqlite::memory:".to_string()))
      .await
      .unwrap()
  }

  mod migration_up_behavior {
    use super::*;

    #[tokio::test]
    async fn should_create_user_table() {
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
      // Verified by successful Entity operations
      use db::models::user;
      let users = user::Entity::find().all(&db).await;
      assert!(users.is_ok());
    }

    #[tokio::test]
    async fn should_create_table_with_primary_key() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;
      migration
        .up(&SchemaManager::new(&db))
        .await
        .expect("migrate");

      // When: inserting user with id
      // Then: should succeed (primary key constraint)
      use chrono::Utc;
      use db::models::user;
      use sea_orm::Set;
      use uuid::Uuid;

      let result = user::Entity::insert(user::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set("test@example.com".to_string()),
        password_hash: Set("hash".to_string()),
        is_active: Set(true),
        is_admin: Set(false),
        current_org_id: Set(None),
        current_role: Set(None),
        created_at: Set(Utc::now().naive_utc()),
        updated_at: Set(Utc::now().naive_utc()),
      })
      .exec(&db)
      .await;

      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_enforce_unique_email() {
      // Given: a test database with user table
      let db = test_db().await;
      let migration = Migration;
      migration
        .up(&SchemaManager::new(&db))
        .await
        .expect("migrate");

      use chrono::Utc;
      use db::models::user;
      use sea_orm::Set;
      use uuid::Uuid;

      let email = "unique@example.com".to_string();
      let now = Utc::now().naive_utc();

      // When: inserting user with email
      user::Entity::insert(user::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(email.clone()),
        password_hash: Set("hash".to_string()),
        is_active: Set(true),
        is_admin: Set(false),
        current_org_id: Set(None),
        current_role: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await
      .expect("insert first");

      // Then: inserting duplicate email should fail
      let result = user::Entity::insert(user::ActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(email),
        password_hash: Set("hash2".to_string()),
        is_active: Set(true),
        is_admin: Set(false),
        current_org_id: Set(None),
        current_role: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await;

      assert!(result.is_err());
    }
  }

  mod migration_down_behavior {
    use super::*;

    #[tokio::test]
    async fn should_drop_user_table() {
      // Given: a test database with user table created
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
      // Given: a test database with user table
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

      // When: trying to query user table
      // Then: should fail (table doesn't exist)
      use db::models::user;
      let result = user::Entity::find().all(&db).await;
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
      assert_eq!(name, "m20220101_000001_create_user_table");
    }
  }
}
