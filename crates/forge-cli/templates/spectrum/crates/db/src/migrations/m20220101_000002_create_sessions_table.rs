use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000002_create_sessions_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Alias::new("sessions"))
          .if_not_exists()
          .col(ColumnDef::new(Alias::new("id")).string().not_null().primary_key())
          .col(ColumnDef::new(Alias::new("data")).binary().not_null())
          .col(ColumnDef::new(Alias::new("expiry_date")).big_integer().not_null())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Alias::new("sessions")).to_owned())
      .await
  }
}
