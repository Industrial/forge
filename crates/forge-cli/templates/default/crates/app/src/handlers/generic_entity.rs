//! Generic REST handler per docs/technical-choices/05-single-generic-rest-handler.md.
//! Single route pattern: entity_id + action; 404 unknown entity; authz and scope from context.
//! Epic 6: expand/include rejected with 400; relations as IDs only.
//! Epic 4: list accepts ListQuerySpec (filter, sort, pagination) via query params.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use db::auth::Backend;
use forge_auth::token_auth::RequireAuth;
use forge_db::DbConnection;
use sea_orm::{EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::Deserialize;
use uuid::Uuid;

use crate::entity_registry;
use crate::handlers::auth::ScopeFromHeaders;
use crate::handlers::dashboard::require_entity_permission;
use crate::handlers::rest::{
  create_organization_impl, delete_organization_impl, update_organization_impl,
  CreateOrganizationBody, UpdateOrganizationBody,
};
use crate::query_spec::{
  FilterOperator, ListQuerySpec, SortDirection, DEFAULT_LIMIT, FilterCond, SortSpec,
  validate_filter_cond, validate_offset_limit, validate_sort_field,
};
use crate::Error as ForgeError;
use db::models::{organization, user};

/// Query params for list: expand/include (rejected), plus filter/sort/pagination (Epic 4).
#[derive(Debug, Deserialize, Default)]
pub struct ListQueryParams {
  #[serde(default)]
  pub expand: Option<String>,
  #[serde(default)]
  pub include: Option<String>,
  #[serde(default)]
  pub filter: Option<String>,
  #[serde(default)]
  pub sort: Option<String>,
  #[serde(default)]
  pub order: Option<String>,
  #[serde(default)]
  pub offset: Option<u64>,
  #[serde(default)]
  pub limit: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct RawFilterCond {
  field: String,
  operator: String,
  value: Option<serde_json::Value>,
}

/// Parse list query params into ListQuerySpec. Returns Err(message) for invalid filter/sort/offset/limit.
/// Public so dashboard (and other legacy list handlers) can adopt ListQuerySpec.
pub fn parse_list_query_spec(params: &ListQueryParams) -> Result<ListQuerySpec, String> {
  let mut spec = ListQuerySpec::default();

  if let Some(ref s) = params.filter {
    if !s.trim().is_empty() {
      let raw: Vec<RawFilterCond> =
        serde_json::from_str(s).map_err(|e| format!("invalid filter JSON: {}", e))?;
      for r in raw {
        let op = FilterOperator::from_str(&r.operator)
          .ok_or_else(|| format!("invalid filter operator: {}", r.operator))?;
        spec.filter.push(FilterCond {
          field: r.field,
          operator: op,
          value: r.value,
        });
      }
    }
  }

  if let Some(ref field) = params.sort {
    if !field.trim().is_empty() {
      let direction = params
        .order
        .as_deref()
        .and_then(SortDirection::from_str)
        .unwrap_or(SortDirection::Asc);
      spec.sort = Some(SortSpec {
        field: field.clone(),
        direction,
      });
    }
  }

  let offset = params.offset.unwrap_or(0);
  let limit = params.limit.unwrap_or(DEFAULT_LIMIT);
  spec.offset_limit = Some(validate_offset_limit(offset, limit)?);

  Ok(spec)
}

fn reject_expand_include(params: &ListQueryParams) -> Option<axum::response::Response> {
  if params.expand.is_some() || params.include.is_some() {
    Some(
      (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({
          "error": "Bad Request",
          "message": "expand and include are not supported; use separate list/get requests for related data"
        })),
      )
        .into_response(),
    )
  } else {
    None
  }
}

/// GET /api/entities/:entity_id — list entities. Query spec (filter, sort, pagination) via query params.
/// 404 unknown entity_id; 403 missing entity.read; scope from headers. Rejects expand/include (400).
/// Query params: filter (JSON array), sort, order, offset, limit. Validates filter/sort against entity allowed fields (400).
pub async fn list_entities(
  Path(entity_id): Path<String>,
  Query(params): Query<ListQueryParams>,
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = reject_expand_include(&params) {
    return Ok(resp);
  }
  if entity_registry::get_entity(entity_id.as_str()).is_none() {
    return Ok((
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({
        "error": "Not Found",
        "message": "Unknown entity"
      })),
    )
      .into_response());
  }
  let user = &auth.0;
  if let Some(resp) =
    require_entity_permission(user, &db, Some(&scope), entity_id.as_str(), "read").await
  {
    return Ok(resp);
  }
  let spec = match parse_list_query_spec(&params) {
    Ok(s) => s,
    Err(msg) => {
      return Ok((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": "Bad Request", "message": msg })),
      )
        .into_response());
    }
  };
  let allowed_filter = entity_registry::effective_filter_fields(entity_id.as_str());
  let allowed_sort = entity_registry::effective_sort_fields(entity_id.as_str());
  for cond in &spec.filter {
    if let Err(msg) = validate_filter_cond(cond, allowed_filter) {
      return Ok((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": "Bad Request", "message": msg })),
      )
        .into_response());
    }
  }
  if let Some(ref sort) = spec.sort {
    if let Err(msg) = validate_sort_field(&sort.field, allowed_sort) {
      return Ok((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": "Bad Request", "message": msg })),
      )
        .into_response());
    }
  }
  match entity_id.as_str() {
    "organization" => list_organizations_impl(&db, &spec)
      .await
      .map(|v| Json(v).into_response()),
    _ => Ok((
      StatusCode::NOT_IMPLEMENTED,
      Json(serde_json::json!({
        "error": "Not Implemented",
        "message": "List not implemented for this entity"
      })),
    )
      .into_response()),
  }
}

fn apply_organization_filter(
  select: sea_orm::Select<organization::Entity>,
  cond: &FilterCond,
) -> sea_orm::Select<organization::Entity> {
  use sea_orm::ColumnTrait;
  match cond.field.as_str() {
    "id" => {
      let parse_uuid = |j: &serde_json::Value| {
        j.as_str().and_then(|s| Uuid::parse_str(s).ok())
      };
      match cond.operator {
        FilterOperator::Eq => {
          if let Some(v) = cond.value.as_ref().and_then(parse_uuid) {
            select.filter(organization::Column::Id.eq(v))
          } else {
            select
          }
        }
        FilterOperator::Ne => {
          if let Some(v) = cond.value.as_ref().and_then(parse_uuid) {
            select.filter(organization::Column::Id.ne(v))
          } else {
            select
          }
        }
        FilterOperator::In => {
          if let Some(serde_json::Value::Array(arr)) = cond.value.as_ref() {
            let vals: Vec<Uuid> = arr.iter().filter_map(parse_uuid).collect();
            if vals.is_empty() {
              select.filter(organization::Column::Id.eq(Uuid::nil()))
            } else {
              select.filter(organization::Column::Id.is_in(vals))
            }
          } else {
            select
          }
        }
        FilterOperator::IsNull => select.filter(organization::Column::Id.is_null()),
        _ => select,
      }
    }
    "name" | "slug" => {
      let col = if cond.field == "name" {
        organization::Column::Name
      } else {
        organization::Column::Slug
      };
      match cond.operator {
        FilterOperator::Eq => {
          if let Some(serde_json::Value::String(s)) = cond.value.as_ref() {
            select.filter(col.eq(s.as_str()))
          } else {
            select
          }
        }
        FilterOperator::Ne => {
          if let Some(serde_json::Value::String(s)) = cond.value.as_ref() {
            select.filter(col.ne(s.as_str()))
          } else {
            select
          }
        }
        FilterOperator::Contains => {
          if let Some(serde_json::Value::String(s)) = cond.value.as_ref() {
            select.filter(col.contains(s))
          } else {
            select
          }
        }
        FilterOperator::StartsWith => {
          if let Some(serde_json::Value::String(s)) = cond.value.as_ref() {
            select.filter(col.starts_with(s))
          } else {
            select
          }
        }
        FilterOperator::EndsWith => {
          if let Some(serde_json::Value::String(s)) = cond.value.as_ref() {
            select.filter(col.ends_with(s))
          } else {
            select
          }
        }
        FilterOperator::In => {
          if let Some(serde_json::Value::Array(arr)) = cond.value.as_ref() {
            let strs: Vec<&str> = arr.iter().filter_map(|j| j.as_str()).collect();
            if strs.is_empty() {
              select.filter(col.eq(""))
            } else {
              select.filter(col.is_in(strs))
            }
          } else {
            select
          }
        }
        FilterOperator::IsNull => select.filter(col.is_null()),
        _ => select,
      }
    }
    "created_at" | "updated_at" => {
      let col = if cond.field == "created_at" {
        organization::Column::CreatedAt
      } else {
        organization::Column::UpdatedAt
      };
      let parse_dt = |j: &serde_json::Value| {
        let s = j.as_str()?;
        let s_trim = s.trim_end_matches('Z');
        chrono::NaiveDateTime::parse_from_str(s_trim, "%Y-%m-%dT%H:%M:%S%.f").ok()
          .or_else(|| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
      };
      match cond.operator {
        FilterOperator::Eq => {
          if let Some(v) = cond.value.as_ref().and_then(parse_dt) {
            select.filter(col.eq(v))
          } else {
            select
          }
        }
        FilterOperator::Ne => {
          if let Some(v) = cond.value.as_ref().and_then(parse_dt) {
            select.filter(col.ne(v))
          } else {
            select
          }
        }
        FilterOperator::IsNull => select.filter(col.is_null()),
        _ => select,
      }
    }
    _ => select,
  }
}

async fn list_organizations_impl(
  db: &DbConnection,
  spec: &ListQuerySpec,
) -> Result<serde_json::Value, ForgeError> {
  let mut select = organization::Entity::find();
  for cond in &spec.filter {
    select = apply_organization_filter(select, cond);
  }

  if let Some(ref sort) = spec.sort {
    let (col, dir) = match sort.field.as_str() {
      "id" => (organization::Column::Id, sort.direction),
      "name" => (organization::Column::Name, sort.direction),
      "slug" => (organization::Column::Slug, sort.direction),
      "created_at" => (organization::Column::CreatedAt, sort.direction),
      "updated_at" => (organization::Column::UpdatedAt, sort.direction),
      _ => (organization::Column::Name, sort.direction),
    };
    select = match dir {
      SortDirection::Asc => select.order_by_asc(col),
      SortDirection::Desc => select.order_by_desc(col),
    };
  } else {
    select = select.order_by_asc(organization::Column::Name);
  }

  let offset = spec.effective_offset();
  let limit = spec.effective_limit();
  select = select.offset(offset).limit(limit);

  let rows = select
    .all(db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  let list: Vec<serde_json::Value> = rows
    .into_iter()
    .map(|r| {
      serde_json::json!({
        "id": r.id.to_string(),
        "name": r.name,
        "slug": r.slug,
        "created_at": r.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
        "updated_at": r.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
      })
    })
    .collect();
  Ok(serde_json::json!({ "data": list }))
}

/// List entity by id with query spec (filter, sort, pagination). Validates spec against entity allowed fields.
/// Returns the list result payload (e.g. `{ "data": [...] }`) for use by REST or RPC.
pub async fn list_entity_with_spec(
  db: &DbConnection,
  entity_id: &str,
  spec: &ListQuerySpec,
) -> Result<serde_json::Value, ForgeError> {
  let allowed_filter = entity_registry::effective_filter_fields(entity_id);
  let allowed_sort = entity_registry::effective_sort_fields(entity_id);
  for cond in &spec.filter {
    validate_filter_cond(cond, allowed_filter)
      .map_err(|msg| ForgeError::Auth(StatusCode::BAD_REQUEST, msg))?;
  }
  if let Some(ref sort) = spec.sort {
    validate_sort_field(&sort.field, allowed_sort)
      .map_err(|msg| ForgeError::Auth(StatusCode::BAD_REQUEST, msg))?;
  }
  match entity_id {
    "organization" => list_organizations_impl(db, spec).await,
    _ => Err(ForgeError::Generic("list not implemented for this entity".to_string())),
  }
}

/// GET /api/entities/:entity_id/:id — get one entity by id. 404 unknown entity or not found. Rejects expand/include (400).
pub async fn get_entity_by_id(
  Path((entity_id, id_str)): Path<(String, String)>,
  Query(params): Query<ListQueryParams>,
  ScopeFromHeaders(scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  if let Some(resp) = reject_expand_include(&params) {
    return Ok(resp);
  }
  if entity_registry::get_entity(entity_id.as_str()).is_none() {
    return Ok((
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({
        "error": "Not Found",
        "message": "Unknown entity"
      })),
    )
      .into_response());
  }
  let user = &auth.0;
  if let Some(resp) =
    require_entity_permission(user, &db, Some(&scope), entity_id.as_str(), "read").await
  {
    return Ok(resp);
  }
  let id = match Uuid::parse_str(&id_str) {
    Ok(u) => u,
    Err(_) => {
      return Ok((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": "Bad Request", "message": "Invalid id" })),
      )
        .into_response());
    }
  };
  match entity_id.as_str() {
    "organization" => get_organization_impl(&db, id).await,
    _ => Ok((
      StatusCode::NOT_IMPLEMENTED,
      Json(serde_json::json!({
        "error": "Not Implemented",
        "message": "Get not implemented for this entity"
      })),
    )
      .into_response()),
  }
}

/// POST /api/entities/:entity_id — create entity. 404 unknown entity; 403 missing entity.create.
pub async fn create_entity(
  Path(entity_id): Path<String>,
  ScopeFromHeaders(_scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, ForgeError> {
  if entity_registry::get_entity(entity_id.as_str()).is_none() {
    return Ok((
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({
        "error": "Not Found",
        "message": "Unknown entity"
      })),
    )
      .into_response());
  }
  let user = &auth.0;
  if let Some(resp) =
    require_entity_permission(user, &db, Some(&_scope), entity_id.as_str(), "create").await
  {
    return Ok(resp);
  }
  match entity_id.as_str() {
    "organization" => {
      let payload: CreateOrganizationBody = match serde_json::from_value(body) {
        Ok(p) => p,
        Err(e) => {
          return Ok((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Bad Request", "message": e.to_string() })),
          )
            .into_response());
        }
      };
      match create_organization_impl(&db, &payload).await {
        Ok(id) => {
          let mut resp = get_organization_impl(&db, id).await?;
          *resp.status_mut() = StatusCode::CREATED;
          Ok(resp)
        }
        Err(ForgeError::Generic(msg)) => Ok((
          StatusCode::UNPROCESSABLE_ENTITY,
          Json(serde_json::json!({ "error": "Unprocessable Entity", "message": msg })),
        )
          .into_response()),
        Err(e) => Err(e),
      }
    }
    _ => Ok((
      StatusCode::NOT_IMPLEMENTED,
      Json(serde_json::json!({
        "error": "Not Implemented",
        "message": "Create not implemented for this entity"
      })),
    )
      .into_response()),
  }
}

/// PATCH /api/entities/:entity_id/:id — update entity. 404 unknown entity or not found; 403 missing entity.update.
pub async fn update_entity(
  Path((entity_id, id_str)): Path<(String, String)>,
  ScopeFromHeaders(_scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
  Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, ForgeError> {
  if entity_registry::get_entity(entity_id.as_str()).is_none() {
    return Ok((
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({
        "error": "Not Found",
        "message": "Unknown entity"
      })),
    )
      .into_response());
  }
  let user = &auth.0;
  if let Some(resp) =
    require_entity_permission(user, &db, Some(&_scope), entity_id.as_str(), "update").await
  {
    return Ok(resp);
  }
  let id = match Uuid::parse_str(&id_str) {
    Ok(u) => u,
    Err(_) => {
      return Ok((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": "Bad Request", "message": "Invalid id" })),
      )
        .into_response());
    }
  };
  match entity_id.as_str() {
    "organization" => {
      let payload: UpdateOrganizationBody = match serde_json::from_value(body) {
        Ok(p) => p,
        Err(e) => {
          return Ok((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Bad Request", "message": e.to_string() })),
          )
            .into_response());
        }
      };
      match update_organization_impl(&db, id, &payload).await {
        Ok(updated) => Ok(Json(serde_json::json!({
          "id": updated.id.to_string(),
          "name": updated.name,
          "slug": updated.slug,
          "created_at": updated.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
          "updated_at": updated.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
        })).into_response()),
        Err(ForgeError::Generic(msg)) if msg.contains("not found") => Ok((
          StatusCode::NOT_FOUND,
          Json(serde_json::json!({ "error": "Not Found", "message": msg })),
        )
          .into_response()),
        Err(e) => Err(e),
      }
    }
    _ => Ok((
      StatusCode::NOT_IMPLEMENTED,
      Json(serde_json::json!({
        "error": "Not Implemented",
        "message": "Update not implemented for this entity"
      })),
    )
      .into_response()),
  }
}

/// DELETE /api/entities/:entity_id/:id — delete entity. 404 unknown entity or not found; 403 missing entity.delete.
pub async fn delete_entity(
  Path((entity_id, id_str)): Path<(String, String)>,
  ScopeFromHeaders(_scope): ScopeFromHeaders,
  auth: RequireAuth<Backend, user::Model>,
  State(db): State<DbConnection>,
) -> Result<impl IntoResponse, ForgeError> {
  if entity_registry::get_entity(entity_id.as_str()).is_none() {
    return Ok((
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({
        "error": "Not Found",
        "message": "Unknown entity"
      })),
    )
      .into_response());
  }
  let user = &auth.0;
  if let Some(resp) =
    require_entity_permission(user, &db, Some(&_scope), entity_id.as_str(), "delete").await
  {
    return Ok(resp);
  }
  let id = match Uuid::parse_str(&id_str) {
    Ok(u) => u,
    Err(_) => {
      return Ok((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": "Bad Request", "message": "Invalid id" })),
      )
        .into_response());
    }
  };
  match entity_id.as_str() {
    "organization" => match delete_organization_impl(&db, id).await {
      Ok(true) => Ok(Json(serde_json::json!({ "ok": true })).into_response()),
      Ok(false) => Ok((
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Not Found", "message": "Resource not found" })),
      )
        .into_response()),
      Err(e) => Err(e),
    },
    _ => Ok((
      StatusCode::NOT_IMPLEMENTED,
      Json(serde_json::json!({
        "error": "Not Implemented",
        "message": "Delete not implemented for this entity"
      })),
    )
      .into_response()),
  }
}

async fn get_organization_impl(
  db: &DbConnection,
  id: Uuid,
) -> Result<axum::response::Response, ForgeError> {
  let row = organization::Entity::find_by_id(id)
    .one(db)
    .await
    .map_err(|e| ForgeError::Generic(e.to_string()))?;
  match row {
    Some(r) => Ok(Json(serde_json::json!({
      "id": r.id.to_string(),
      "name": r.name,
      "slug": r.slug,
      "created_at": r.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
      "updated_at": r.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
    })).into_response()),
    None => Ok((
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({ "error": "Not Found", "message": "Resource not found" })),
    )
      .into_response()),
  }
}
