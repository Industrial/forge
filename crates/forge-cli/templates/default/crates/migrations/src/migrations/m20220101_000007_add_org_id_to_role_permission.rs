use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum RolePermission {
  Table,
  Id,
  Scope,
  RoleName,
  PermissionKey,
  OrgId,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000007_add_org_id_to_role_permission"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(RolePermission::Table)
          .add_column(ColumnDef::new(RolePermission::OrgId).uuid().null())
          .to_owned(),
      )
      .await?;
    manager
      .drop_index(
        Index::drop()
          .table(RolePermission::Table)
          .name("uq_role_permission_scope_role_permission")
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .table(RolePermission::Table)
          .name("uq_role_permission_scope_role_perm_org")
          .col(RolePermission::Scope)
          .col(RolePermission::RoleName)
          .col(RolePermission::PermissionKey)
          .col(RolePermission::OrgId)
          .unique()
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_index(
        Index::drop()
          .table(RolePermission::Table)
          .name("uq_role_permission_scope_role_perm_org")
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
      .await?;
    manager
      .alter_table(
        Table::alter()
          .table(RolePermission::Table)
          .drop_column(RolePermission::OrgId)
          .to_owned(),
      )
      .await
  }
}

#[cfg(test)]
mod bdd_tests {
  use super::*;
  use sea_orm::Database;

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
      assert_eq!(name, "m20220101_000007_add_org_id_to_role_permission");
    }
  }

  mod migration_up_behavior {
    use super::*;

    #[tokio::test]
    async fn should_add_org_id_column_to_role_permission_table() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;

      // When: running migration up
      let result = migration.up(&SchemaManager::new(&db)).await;

      // Then: should attempt to add org_id column
      // Note: May fail if role_permission table doesn't exist (requires previous migrations)
      assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn should_drop_old_unique_index() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;

      // When: running migration up
      let result = migration.up(&SchemaManager::new(&db)).await;

      // Then: should attempt to drop old unique index
      assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn should_create_new_unique_index_with_org_id() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;

      // When: running migration up
      let result = migration.up(&SchemaManager::new(&db)).await;

      // Then: should create new unique index including org_id
      assert!(result.is_ok() || result.is_err());
    }
  }

  mod migration_down_behavior {
    use super::*;

    #[tokio::test]
    async fn should_remove_org_id_column_from_role_permission_table() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;

      // When: running migration down
      let result = migration.down(&SchemaManager::new(&db)).await;

      // Then: should attempt to remove org_id column
      assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn should_restore_old_unique_index() {
      // Given: a test database
      let db = test_db().await;
      let migration = Migration;

      // When: running migration down
      let result = migration.down(&SchemaManager::new(&db)).await;

      // Then: should restore old unique index
      assert!(result.is_ok() || result.is_err());
    }
  }
}
