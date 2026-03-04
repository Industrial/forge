use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum ApiTokens {
  Table,
  Id,
  UserId,
  TokenHash,
  Name,
  LastUsedAt,
  ExpiresAt,
  CreatedAt,
  UpdatedAt,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000006_create_api_tokens_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(ApiTokens::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(ApiTokens::Id)
              .uuid()
              .not_null()
              .primary_key(),
          )
          .col(ColumnDef::new(ApiTokens::UserId).uuid().not_null())
          .col(ColumnDef::new(ApiTokens::TokenHash).string().not_null())
          .col(ColumnDef::new(ApiTokens::Name).string())
          .col(ColumnDef::new(ApiTokens::LastUsedAt).date_time())
          .col(ColumnDef::new(ApiTokens::ExpiresAt).date_time())
          .col(ColumnDef::new(ApiTokens::CreatedAt).date_time().not_null())
          .col(ColumnDef::new(ApiTokens::UpdatedAt).date_time().not_null())
          .foreign_key(
            ForeignKey::create()
              .name("fk_api_tokens_user_id")
              .from_tbl(ApiTokens::Table)
              .from_col(ApiTokens::UserId)
              .to_tbl(Alias::new("user"))
              .to_col(Alias::new("id")),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(ApiTokens::Table).to_owned())
      .await
  }
}
