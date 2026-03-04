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
    "m20220101_000004_create_memberships_table"
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
