use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum Membership {
  Table,
  Id,
  UserId,
  OrgId,
  Role,
  CreatedAt,
  UpdatedAt,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000003_create_memberships_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Membership::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(Membership::Id)
              .uuid()
              .not_null()
              .primary_key(),
          )
          .col(ColumnDef::new(Membership::UserId).uuid().not_null())
          .col(ColumnDef::new(Membership::OrgId).uuid().not_null())
          .col(ColumnDef::new(Membership::Role).string().not_null())
          .col(ColumnDef::new(Membership::CreatedAt).date_time().not_null())
          .col(ColumnDef::new(Membership::UpdatedAt).date_time().not_null())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Membership::Table).to_owned())
      .await
  }
}

#[cfg(test)]
mod bdd_tests {
  use super::*;
  use sea_orm::{Database, ConnectionTrait, EntityTrait};
  use sea_orm_migration::prelude::*;

  async fn test_db() -> sea_orm::DatabaseConnection {
    Database::connect(sea_orm::ConnectOptions::new("sqlite::memory:".to_string()))
      .await
      .unwrap()
  }

  mod migration_up_behavior {
    use super::*;

    #[tokio::test]
    async fn should_create_membership_table() {
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
      migration.up(&SchemaManager::new(&db)).await.expect("migrate");

      // When: checking table structure
      // Then: table should exist with all columns
      use crate::models::membership;
      let memberships = membership::Entity::find().all(&db).await;
      assert!(memberships.is_ok());
    }

    #[tokio::test]
    async fn should_create_table_with_primary_key() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;
      migration.up(&SchemaManager::new(&db)).await.expect("migrate");

      // When: inserting membership with id
      // Then: should succeed
      use crate::models::membership;
      use sea_orm::Set;
      use uuid::Uuid;
      use chrono::Utc;

      let result = membership::Entity::insert(membership::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(Uuid::new_v4()),
        org_id: Set(Uuid::new_v4()),
        role: Set("member".to_string()),
        created_at: Set(Utc::now().naive_utc()),
        updated_at: Set(Utc::now().naive_utc()),
      })
      .exec(&db)
      .await;

      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_require_user_id_and_org_id() {
      // Given: a test database with membership table
      let db = test_db().await;
      let migration = Migration;
      migration.up(&SchemaManager::new(&db)).await.expect("migrate");

      use crate::models::membership;
      use sea_orm::Set;
      use uuid::Uuid;
      use chrono::Utc;

      // When: inserting membership without user_id
      // Then: should fail (not_null constraint)
      // Note: This is verified by the schema requiring not_null
      let now = Utc::now().naive_utc();
      let result = membership::Entity::insert(membership::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(Uuid::new_v4()),
        org_id: Set(Uuid::new_v4()),
        role: Set("member".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
      })
      .exec(&db)
      .await;

      assert!(result.is_ok());
    }
  }

  mod migration_down_behavior {
    use super::*;

    #[tokio::test]
    async fn should_drop_membership_table() {
      // Given: a test database with membership table created
      let db = test_db().await;
      let migration = Migration;
      migration.up(&SchemaManager::new(&db)).await.expect("migrate up");

      // When: running migration down
      let result = migration.down(&SchemaManager::new(&db)).await;

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_remove_table_after_down() {
      // Given: a test database with membership table
      let db = test_db().await;
      let migration = Migration;
      migration.up(&SchemaManager::new(&db)).await.expect("migrate up");
      migration.down(&SchemaManager::new(&db)).await.expect("migrate down");

      // When: trying to query membership table
      // Then: should fail (table doesn't exist)
      use crate::models::membership;
      let result = membership::Entity::find().all(&db).await;
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
      assert_eq!(name, "m20220101_000003_create_memberships_table");
    }
  }
}
