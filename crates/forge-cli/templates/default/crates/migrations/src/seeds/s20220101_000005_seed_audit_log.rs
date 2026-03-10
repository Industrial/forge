//! Seed audit log entries for the Default org so e2e tests (e.g. pagination) have multiple pages.

use forge_db::DbConnection;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};

use db::models::{audit_log, organization, user};

/// Enough for 2+ pages at any default (10, 25, or 50) so e2e "next" is enabled.
const NUM_ENTRIES: u32 = 55;

pub async fn seed(db: &DbConnection) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let org = organization::Entity::find()
    .filter(organization::Column::Slug.eq("default"))
    .one(db)
    .await?
    .ok_or("Default org not found")?;

  let viewer = user::Entity::find()
    .filter(user::Column::Email.eq("viewer@default.org"))
    .one(db)
    .await?
    .ok_or("viewer@default.org not found")?;

  let now = chrono::Utc::now().naive_utc();
  for i in 0..NUM_ENTRIES {
    let occurred = now - chrono::Duration::days(i as i64);
    audit_log::Entity::insert(audit_log::ActiveModel {
      id: Set(uuid::Uuid::new_v4()),
      event_kind: Set("auth".to_string()),
      actor_id: Set(viewer.id),
      subject_id: Set(None),
      organization_id: Set(Some(org.id)),
      action: Set("read".to_string()),
      resource_type: Set("audit_log".to_string()),
      resource_id: Set(None),
      outcome: Set("success".to_string()),
      reason: Set(Some(format!("Seed entry {}", i + 1))),
      occurred_at: Set(occurred),
    })
    .exec(db)
    .await?;
  }

  Ok(())
}
