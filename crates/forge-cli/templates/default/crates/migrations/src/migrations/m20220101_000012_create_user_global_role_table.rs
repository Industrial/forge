use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum UserGlobalRole {
  Table,
  Id,
  UserId,
  RoleName,
}

#[derive(Iden)]
enum User {
  Table,
  Id,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000012_create_user_global_role_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(UserGlobalRole::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(UserGlobalRole::Id)
              .uuid()
              .not_null()
              .primary_key(),
          )
          .col(ColumnDef::new(UserGlobalRole::UserId).uuid().not_null())
          .col(ColumnDef::new(UserGlobalRole::RoleName).string().not_null())
          .foreign_key(
            ForeignKey::create()
              .name("fk_user_global_role_user_id")
              .from(UserGlobalRole::Table, UserGlobalRole::UserId)
              .to(User::Table, User::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .table(UserGlobalRole::Table)
          .name("uq_user_global_role_user_id_role_name")
          .col(UserGlobalRole::UserId)
          .col(UserGlobalRole::RoleName)
          .unique()
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .table(UserGlobalRole::Table)
          .name("ix_user_global_role_user_id")
          .col(UserGlobalRole::UserId)
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(UserGlobalRole::Table).to_owned())
      .await
  }
}
