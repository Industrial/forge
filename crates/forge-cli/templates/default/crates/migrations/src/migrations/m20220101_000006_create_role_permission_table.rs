use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum RolePermission {
  Table,
  Id,
  Scope,
  RoleName,
  PermissionKey,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000006_create_role_permission_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(RolePermission::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(RolePermission::Id)
              .uuid()
              .not_null()
              .primary_key(),
          )
          .col(ColumnDef::new(RolePermission::Scope).string().not_null())
          .col(ColumnDef::new(RolePermission::RoleName).string().not_null())
          .col(
            ColumnDef::new(RolePermission::PermissionKey)
              .string()
              .not_null(),
          )
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .table(RolePermission::Table)
          .name("uq_role_permission_scope_role_permission")
          .col(RolePermission::Scope)
          .col(RolePermission::RoleName)
          .col(RolePermission::PermissionKey)
          .unique()
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(RolePermission::Table).to_owned())
      .await
  }
}

#[cfg(test)]
mod bdd_tests {
  use super::*;
  use sea_orm::{Database, EntityTrait};

  async fn test_db() -> sea_orm::DatabaseConnection {
    Database::connect(sea_orm::ConnectOptions::new("sqlite::memory:".to_string()))
      .await
      .unwrap()
  }

  mod migration_up_behavior {
    use super::*;

    #[tokio::test]
    async fn should_create_role_permission_table() {
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
      // Note: Migration 7 adds org_id column, so we need to run that too to use the model
      use crate::migrations::m20220101_000007_add_org_id_to_role_permission;
      let migration7 = m20220101_000007_add_org_id_to_role_permission::Migration;
      migration7
        .up(&SchemaManager::new(&db))
        .await
        .expect("migrate to add org_id");

      use db::models::role_permission;
      let perms = role_permission::Entity::find().all(&db).await;
      assert!(perms.is_ok());
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

      // Run migration 7 to add org_id column (required by model)
      use crate::migrations::m20220101_000007_add_org_id_to_role_permission;
      let migration7 = m20220101_000007_add_org_id_to_role_permission::Migration;
      migration7
        .up(&SchemaManager::new(&db))
        .await
        .expect("migrate to add org_id");

      // When: inserting role_permission with id
      // Then: should succeed
      use db::models::role_permission;
      use sea_orm::Set;
      use uuid::Uuid;

      let result = role_permission::Entity::insert(role_permission::ActiveModel {
        id: Set(Uuid::new_v4()),
        scope: Set("global".to_string()),
        role_name: Set("admin".to_string()),
        permission_key: Set("test.permission".to_string()),
        org_id: Set(None),
      })
      .exec(&db)
      .await;

      assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_create_unique_index() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;

      // When: running migration up
      let result = migration.up(&SchemaManager::new(&db)).await;

      // Then: should succeed (unique index created)
      assert!(result.is_ok());
    }
  }

  mod migration_down_behavior {
    use super::*;

    #[tokio::test]
    async fn should_drop_role_permission_table() {
      // Given: a test database with role_permission table created
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
      // Given: a test database with role_permission table
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

      // When: trying to query role_permission table
      // Then: should fail (table doesn't exist)
      use db::models::role_permission;
      let result = role_permission::Entity::find().all(&db).await;
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
      assert_eq!(name, "m20220101_000006_create_role_permission_table");
    }
  }
}
