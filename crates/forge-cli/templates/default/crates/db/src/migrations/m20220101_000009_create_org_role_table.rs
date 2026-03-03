use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum OrgRole {
  Table,
  Id,
  OrgId,
  Name,
  DisplayName,
  CreatedAt,
  UpdatedAt,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000009_create_org_role_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(OrgRole::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(OrgRole::Id)
              .uuid()
              .not_null()
              .primary_key(),
          )
          .col(ColumnDef::new(OrgRole::OrgId).uuid().not_null())
          .col(ColumnDef::new(OrgRole::Name).string().not_null())
          .col(ColumnDef::new(OrgRole::DisplayName).string().null())
          .col(ColumnDef::new(OrgRole::CreatedAt).date_time().not_null())
          .col(ColumnDef::new(OrgRole::UpdatedAt).date_time().not_null())
          .foreign_key(
            ForeignKey::create()
              .name("fk_org_role_org_id")
              .from(OrgRole::Table, OrgRole::OrgId)
              .to(Organization::Table, Organization::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .table(OrgRole::Table)
          .name("uq_org_role_org_id_name")
          .col(OrgRole::OrgId)
          .col(OrgRole::Name)
          .unique()
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(OrgRole::Table).to_owned())
      .await
  }
}

#[derive(Iden)]
enum Organization {
  Table,
  Id,
}
