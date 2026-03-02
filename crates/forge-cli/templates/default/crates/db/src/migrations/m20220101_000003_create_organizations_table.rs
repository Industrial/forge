use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum Organization {
  Table,
  Id,
  Name,
  Slug,
  CreatedAt,
  UpdatedAt,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000003_create_organizations_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Organization::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(Organization::Id)
              .uuid()
              .not_null()
              .primary_key(),
          )
          .col(ColumnDef::new(Organization::Name).string().not_null())
          .col(
            ColumnDef::new(Organization::Slug)
              .string()
              .unique_key()
              .not_null(),
          )
          .col(
            ColumnDef::new(Organization::CreatedAt)
              .date_time()
              .not_null(),
          )
          .col(
            ColumnDef::new(Organization::UpdatedAt)
              .date_time()
              .not_null(),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Organization::Table).to_owned())
      .await
  }
}
