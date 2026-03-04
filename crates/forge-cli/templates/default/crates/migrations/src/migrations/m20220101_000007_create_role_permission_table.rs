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
    "m20220101_000007_create_role_permission_table"
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
