use sea_orm_migration::prelude::*;

/// Audit log table. Column names must match forge::audit::log raw SQL (snake_case).
#[derive(Iden)]
pub enum AuditLog {
  Table,
  Id,
  #[iden = "event_kind"]
  EventKind,
  #[iden = "actor_id"]
  ActorId,
  #[iden = "subject_id"]
  SubjectId,
  #[iden = "organization_id"]
  OrganizationId,
  Action,
  #[iden = "resource_type"]
  ResourceType,
  #[iden = "resource_id"]
  ResourceId,
  Outcome,
  Reason,
  #[iden = "occurred_at"]
  OccurredAt,
}

pub struct Migration;

impl MigrationName for Migration {
  fn name(&self) -> &str {
    "m20220101_000005_create_audit_log_table"
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(AuditLog::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(AuditLog::Id)
              .uuid()
              .not_null()
              .primary_key(),
          )
          .col(ColumnDef::new(AuditLog::EventKind).string().not_null())
          .col(ColumnDef::new(AuditLog::ActorId).uuid().not_null())
          .col(ColumnDef::new(AuditLog::SubjectId).uuid())
          .col(ColumnDef::new(AuditLog::OrganizationId).uuid())
          .col(ColumnDef::new(AuditLog::Action).string().not_null())
          .col(ColumnDef::new(AuditLog::ResourceType).string().not_null())
          .col(ColumnDef::new(AuditLog::ResourceId).uuid())
          .col(ColumnDef::new(AuditLog::Outcome).string().not_null())
          .col(ColumnDef::new(AuditLog::Reason).string())
          .col(ColumnDef::new(AuditLog::OccurredAt).date_time().not_null())
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_audit_log_org_occurred")
          .table(AuditLog::Table)
          .col(AuditLog::OrganizationId)
          .col(AuditLog::OccurredAt)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_audit_log_actor_occurred")
          .table(AuditLog::Table)
          .col(AuditLog::ActorId)
          .col(AuditLog::OccurredAt)
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(AuditLog::Table).to_owned())
      .await
  }
}
