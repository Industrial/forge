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
    "m20220101_000008_add_org_id_to_role_permission"
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
