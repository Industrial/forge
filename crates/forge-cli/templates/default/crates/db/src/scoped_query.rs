//! Scoped query helpers: apply request scope (current org) to SeaORM selects.
//!
//! Use [WithScope::with_scope] to restrict results to the current tenant/org.
//! For [user](crate::models::user) (scoped via membership), use [user_find_scoped].

use forge_auth::RequestScope;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Select};
use uuid::Uuid;

use crate::models::{audit_log, membership, org_role, role_permission, user, user_org_role};

/// Apply request scope to a select: when `scope_opt` is `Some`, filter to that
/// org; when `None`, leave query unchanged.
pub trait WithScope {
  /// Entity type for the resulting select.
  type Entity: EntityTrait;
  /// Return a select restricted to the scope's organization when `scope_opt` is
  /// `Some`; otherwise return the select unchanged.
  fn with_scope(self, scope_opt: Option<&RequestScope>) -> Select<Self::Entity>;
}

impl WithScope for Select<audit_log::Entity> {
  type Entity = audit_log::Entity;
  fn with_scope(self, scope_opt: Option<&RequestScope>) -> Select<audit_log::Entity> {
    match scope_opt {
      None => self,
      Some(s) => self.filter(audit_log::Column::OrganizationId.eq(s.organization_id)),
    }
  }
}

impl WithScope for Select<org_role::Entity> {
  type Entity = org_role::Entity;
  fn with_scope(self, scope_opt: Option<&RequestScope>) -> Select<org_role::Entity> {
    match scope_opt {
      None => self,
      Some(s) => self.filter(org_role::Column::OrgId.eq(s.organization_id)),
    }
  }
}

impl WithScope for Select<user_org_role::Entity> {
  type Entity = user_org_role::Entity;
  fn with_scope(self, scope_opt: Option<&RequestScope>) -> Select<user_org_role::Entity> {
    match scope_opt {
      None => self,
      Some(s) => self.filter(user_org_role::Column::OrgId.eq(s.organization_id)),
    }
  }
}

impl WithScope for Select<membership::Entity> {
  type Entity = membership::Entity;
  fn with_scope(self, scope_opt: Option<&RequestScope>) -> Select<membership::Entity> {
    match scope_opt {
      None => self,
      Some(s) => self.filter(membership::Column::OrgId.eq(s.organization_id)),
    }
  }
}

impl WithScope for Select<role_permission::Entity> {
  type Entity = role_permission::Entity;
  fn with_scope(self, scope_opt: Option<&RequestScope>) -> Select<role_permission::Entity> {
    match scope_opt {
      None => self,
      Some(s) => self.filter(role_permission::Column::OrgId.eq(Some(s.organization_id))),
    }
  }
}

/// Return a select of users, optionally restricted to those with membership in the scope's org.
pub async fn user_find_scoped<C: ConnectionTrait>(
  db: &C,
  scope_opt: Option<&RequestScope>,
) -> Result<Select<user::Entity>, sea_orm::DbErr> {
  let select = user::Entity::find();
  match scope_opt {
    None => Ok(select),
    Some(scope) => {
      let user_ids: Vec<Uuid> = membership::Entity::find()
        .filter(membership::Column::OrgId.eq(scope.organization_id))
        .all(db)
        .await?
        .into_iter()
        .map(|m| m.user_id)
        .collect();
      Ok(select.filter(user::Column::Id.is_in(user_ids)))
    }
  }
}
