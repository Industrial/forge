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
    "m20220101_000010_create_user_org_role_table"
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
          .col(ColumnDef::new(UserOrgRole::CreatedAt).date_time().not_null())
          .col(ColumnDef::new(UserOrgRole::UpdatedAt).date_time().not_null())
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
