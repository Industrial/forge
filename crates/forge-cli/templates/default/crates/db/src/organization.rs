//! Organization CUD and body types. Used by RestModel impl and by app legacy routes.

use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::Deserialize;
use uuid::Uuid;

use crate::model_error::ModelError;
use crate::models::organization;

fn slug_from_name(name: &str) -> String {
  name
    .to_lowercase()
    .chars()
    .map(|c| {
      if c.is_alphanumeric() || c == ' ' {
        c
      } else {
        '-'
      }
    })
    .collect::<String>()
    .split_whitespace()
    .filter(|s| !s.is_empty())
    .collect::<Vec<_>>()
    .join("-")
}

#[derive(Debug, Deserialize)]
pub struct CreateOrganizationBody {
  pub name: String,
  pub slug: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrganizationBody {
  pub name: Option<String>,
  pub slug: Option<String>,
}

/// Create organization; returns new id. Used by generic handler and seeds.
pub async fn create_organization_impl<C: sea_orm::ConnectionTrait>(
  db: &C,
  payload: &CreateOrganizationBody,
) -> Result<Uuid, ModelError> {
  let name = payload.name.trim();
  if name.is_empty() {
    return Err(ModelError::Validation("name is required".into()));
  }
  let slug = payload
    .slug
    .as_deref()
    .map(|s| s.trim())
    .filter(|s| !s.is_empty())
    .map(String::from)
    .unwrap_or_else(|| slug_from_name(name));
  let now = chrono::Utc::now().naive_utc();
  let id = Uuid::new_v4();
  organization::Entity::insert(organization::ActiveModel {
    id: Set(id),
    name: Set(name.to_string()),
    slug: Set(slug),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  })
  .exec(db)
  .await?;
  Ok(id)
}

/// Idempotent: find organization by slug or create. For use in seeds and get-or-create flows.
pub async fn ensure_organization_impl<C: sea_orm::ConnectionTrait>(
  db: &C,
  payload: &CreateOrganizationBody,
) -> Result<Uuid, ModelError> {
  let name = payload.name.trim();
  if name.is_empty() {
    return Err(ModelError::Validation("name is required".into()));
  }
  let slug = payload
    .slug
    .as_deref()
    .map(|s| s.trim())
    .filter(|s| !s.is_empty())
    .map(String::from)
    .unwrap_or_else(|| slug_from_name(name));
  use sea_orm::{ColumnTrait, QueryFilter};
  if let Some(existing) = organization::Entity::find()
    .filter(organization::Column::Slug.eq(&slug))
    .one(db)
    .await?
  {
    return Ok(existing.id);
  }
  create_organization_impl(db, payload).await
}

/// Update organization by id. Returns the updated model.
pub async fn update_organization_impl<C: sea_orm::ConnectionTrait>(
  db: &C,
  id: Uuid,
  payload: &UpdateOrganizationBody,
) -> Result<organization::Model, ModelError> {
  let o = organization::Entity::find_by_id(id)
    .one(db)
    .await?
    .ok_or_else(|| ModelError::NotFound("Organization not found".into()))?;
  let mut am: organization::ActiveModel = o.into();
  if let Some(n) = &payload.name {
    let t = n.trim();
    if !t.is_empty() {
      am.name = Set(t.to_string());
    }
  }
  if let Some(s) = &payload.slug {
    let t = s.trim();
    if !t.is_empty() {
      am.slug = Set(t.to_string());
    }
  }
  am.updated_at = Set(chrono::Utc::now().naive_utc());
  am.update(db).await?;
  organization::Entity::find_by_id(id)
    .one(db)
    .await?
    .ok_or_else(|| ModelError::NotFound("Organization not found".into()))
}

/// Delete organization by id. Returns true if deleted, false if not found.
pub async fn delete_organization_impl<C: sea_orm::ConnectionTrait>(
  db: &C,
  id: Uuid,
) -> Result<bool, ModelError> {
  let r = organization::Entity::delete_by_id(id).exec(db).await?;
  Ok(r.rows_affected > 0)
}
