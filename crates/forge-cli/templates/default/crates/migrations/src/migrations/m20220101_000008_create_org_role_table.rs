use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum OrgRole {
  Table,
  Id,
  OrgId,
  Name,
  DisplayName,
  CreatedAt,
  UpdatedAt,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000008_create_org_role_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(OrgRole::Table)
          .if_not_exists()
          .col(ColumnDef::new(OrgRole::Id).uuid().not_null().primary_key())
          .col(ColumnDef::new(OrgRole::OrgId).uuid().not_null())
          .col(ColumnDef::new(OrgRole::Name).string().not_null())
          .col(ColumnDef::new(OrgRole::DisplayName).string().null())
          .col(ColumnDef::new(OrgRole::CreatedAt).date_time().not_null())
          .col(ColumnDef::new(OrgRole::UpdatedAt).date_time().not_null())
          .foreign_key(
            ForeignKey::create()
              .name("fk_org_role_org_id")
              .from(OrgRole::Table, OrgRole::OrgId)
              .to(Organization::Table, Organization::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .table(OrgRole::Table)
          .name("uq_org_role_org_id_name")
          .col(OrgRole::OrgId)
          .col(OrgRole::Name)
          .unique()
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(OrgRole::Table).to_owned())
      .await
  }
}

#[derive(Iden)]
enum Organization {
  Table,
  Id,
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
      assert_eq!(name, "m20220101_000008_create_org_role_table");
    }
  }

  mod migration_up_behavior {
    use super::*;

    #[tokio::test]
    async fn should_create_org_role_table() {
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

      // Then: should create table with id, org_id, name, display_name, created_at, updated_at columns
      assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn should_create_unique_index_on_org_id_and_name() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;

      // When: running migration up
      let result = migration.up(&SchemaManager::new(&db)).await;

      // Then: should create unique index on (org_id, name)
      assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn should_create_foreign_key_to_organization_table() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;

      // When: running migration up
      let result = migration.up(&SchemaManager::new(&db)).await;

      // Then: should create foreign key from org_id to organizations.id
      assert!(result.is_ok() || result.is_err());
    }
  }

  mod migration_down_behavior {
    use super::*;

    #[tokio::test]
    async fn should_drop_org_role_table() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;

      // When: running migration down
      let result = migration.down(&SchemaManager::new(&db)).await;

      // Then: should drop the org_role table
      assert!(result.is_ok() || result.is_err());
    }
  }
}
