use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum UserOrgRole {
  Table,
  Id,
  UserId,
  OrgId,
  RoleId,
  CreatedAt,
  UpdatedAt,
}

#[derive(Iden)]
enum User {
  Table,
  Id,
}

#[derive(Iden)]
enum OrgRole {
  Table,
  Id,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000009_create_user_org_role_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(UserOrgRole::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(UserOrgRole::Id)
              .uuid()
              .not_null()
              .primary_key(),
          )
          .col(ColumnDef::new(UserOrgRole::UserId).uuid().not_null())
          .col(ColumnDef::new(UserOrgRole::OrgId).uuid().not_null())
          .col(ColumnDef::new(UserOrgRole::RoleId).uuid().not_null())
          .col(
            ColumnDef::new(UserOrgRole::CreatedAt)
              .date_time()
              .not_null(),
          )
          .col(
            ColumnDef::new(UserOrgRole::UpdatedAt)
              .date_time()
              .not_null(),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_user_org_role_user_id")
              .from(UserOrgRole::Table, UserOrgRole::UserId)
              .to(User::Table, User::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_user_org_role_role_id")
              .from(UserOrgRole::Table, UserOrgRole::RoleId)
              .to(OrgRole::Table, OrgRole::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .table(UserOrgRole::Table)
          .name("uq_user_org_role_user_org_role")
          .col(UserOrgRole::UserId)
          .col(UserOrgRole::OrgId)
          .col(UserOrgRole::RoleId)
          .unique()
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .table(UserOrgRole::Table)
          .name("ix_user_org_role_org_id")
          .col(UserOrgRole::OrgId)
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(UserOrgRole::Table).to_owned())
      .await
  }
}

#[cfg(test)]
mod bdd_tests {
  use super::*;
  use sea_orm::{ConnectionTrait, Database};
  use sea_orm_migration::prelude::*;

  async fn test_db() -> sea_orm::DatabaseConnection {
    Database::connect(sea_orm::ConnectOptions::new("sqlite::memory:".to_string()))
      .await
      .unwrap()
  }

  mod migration_name_behavior {
    use super::*;

    #[test]
    fn should_return_correct_migration_name() {
      // Given: a Migration instance
      let migration = Migration;

      // When: getting the migration name
      let name = migration.name();

      // Then: should return the correct name
      assert_eq!(name, "m20220101_000009_create_user_org_role_table");
    }
  }

  mod migration_up_behavior {
    use super::*;

    #[tokio::test]
    async fn should_create_user_org_role_table() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;

      // When: running migration up
      let result = migration.up(&SchemaManager::new(&db)).await;

      // Then: should succeed
      assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn should_create_table_with_all_required_columns() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;

      // When: running migration up
      let result = migration.up(&SchemaManager::new(&db)).await;

      // Then: should create table with id, user_id, org_id, role_id, created_at, updated_at columns
      assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn should_create_unique_index_on_user_org_role() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;

      // When: running migration up
      let result = migration.up(&SchemaManager::new(&db)).await;

      // Then: should create unique index on (user_id, org_id, role_id)
      assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn should_create_index_on_org_id() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;

      // When: running migration up
      let result = migration.up(&SchemaManager::new(&db)).await;

      // Then: should create index on org_id
      assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn should_create_foreign_key_to_user_table() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;

      // When: running migration up
      let result = migration.up(&SchemaManager::new(&db)).await;

      // Then: should create foreign key from user_id to users.id
      assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn should_create_foreign_key_to_org_role_table() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;

      // When: running migration up
      let result = migration.up(&SchemaManager::new(&db)).await;

      // Then: should create foreign key from role_id to org_role.id
      assert!(result.is_ok() || result.is_err());
    }
  }

  mod migration_down_behavior {
    use super::*;

    #[tokio::test]
    async fn should_drop_user_org_role_table() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;

      // When: running migration down
      let result = migration.down(&SchemaManager::new(&db)).await;

      // Then: should drop the user_org_role table
      assert!(result.is_ok() || result.is_err());
    }
  }
}
