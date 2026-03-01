use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum User {
  Table,
  Id,
  Email,
  PasswordHash,
  IsActive,
  IsAdmin,
  CurrentOrgId,
  CurrentRole,
  CreatedAt,
  UpdatedAt,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000001_create_user_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(User::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(User::Id)
              .uuid()
              .not_null()
              .primary_key(),
          )
          .col(ColumnDef::new(User::Email).string().unique_key().not_null())
          .col(ColumnDef::new(User::PasswordHash).string().not_null())
          .col(ColumnDef::new(User::IsActive).boolean().not_null().default(true))
          .col(ColumnDef::new(User::IsAdmin).boolean().not_null().default(false))
          .col(ColumnDef::new(User::CurrentOrgId).uuid())
          .col(ColumnDef::new(User::CurrentRole).string())
          .col(ColumnDef::new(User::CreatedAt).date_time().not_null())
          .col(ColumnDef::new(User::UpdatedAt).date_time().not_null())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(User::Table).to_owned())
      .await
  }
}
