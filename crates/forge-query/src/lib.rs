//! Unified query specification: filter, sort, and pagination types with validation.
//! Scope is applied from request context only, not part of the spec.

use serde::{Deserialize, Serialize};

/// Filter operators (§2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterOperator {
  Eq,
  Ne,
  Gt,
  Gte,
  Lt,
  Lte,
  In,
  Contains,
  StartsWith,
  EndsWith,
  IsNull,
}

impl FilterOperator {
  pub const ALL: &'static [FilterOperator] = &[
    FilterOperator::Eq,
    FilterOperator::Ne,
    FilterOperator::Gt,
    FilterOperator::Gte,
    FilterOperator::Lt,
    FilterOperator::Lte,
    FilterOperator::In,
    FilterOperator::Contains,
    FilterOperator::StartsWith,
    FilterOperator::EndsWith,
    FilterOperator::IsNull,
  ];

  pub fn as_str(self) -> &'static str {
    match self {
      FilterOperator::Eq => "eq",
      FilterOperator::Ne => "ne",
      FilterOperator::Gt => "gt",
      FilterOperator::Gte => "gte",
      FilterOperator::Lt => "lt",
      FilterOperator::Lte => "lte",
      FilterOperator::In => "in",
      FilterOperator::Contains => "contains",
      FilterOperator::StartsWith => "starts_with",
      FilterOperator::EndsWith => "ends_with",
      FilterOperator::IsNull => "is_null",
    }
  }

  pub fn from_str(s: &str) -> Option<FilterOperator> {
    match s {
      "eq" => Some(FilterOperator::Eq),
      "ne" => Some(FilterOperator::Ne),
      "gt" => Some(FilterOperator::Gt),
      "gte" => Some(FilterOperator::Gte),
      "lt" => Some(FilterOperator::Lt),
      "lte" => Some(FilterOperator::Lte),
      "in" => Some(FilterOperator::In),
      "contains" => Some(FilterOperator::Contains),
      "starts_with" => Some(FilterOperator::StartsWith),
      "ends_with" => Some(FilterOperator::EndsWith),
      "is_null" => Some(FilterOperator::IsNull),
      _ => None,
    }
  }
}

/// A single filter condition: field + operator + optional value (§2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCond {
  pub field: String,
  pub operator: FilterOperator,
  /// None for is_null; single value for most; vec for `in`.
  pub value: Option<serde_json::Value>,
}

/// Validates that `field` is in `allowed_fields`. Returns `Ok(())` or `Err(message)`.
pub fn validate_filter_field(field: &str, allowed_fields: &[&str]) -> Result<(), String> {
  if allowed_fields.is_empty() {
    return Err("no filter fields allowed for this entity".to_string());
  }
  if allowed_fields.contains(&field) {
    Ok(())
  } else {
    Err(format!(
      "invalid filter field '{}'; allowed: {}",
      field,
      allowed_fields.join(", ")
    ))
  }
}

/// Validates a filter condition against allowed fields.
pub fn validate_filter_cond(cond: &FilterCond, allowed_fields: &[&str]) -> Result<(), String> {
  validate_filter_field(cond.field.as_str(), allowed_fields)?;
  match cond.operator {
    FilterOperator::IsNull => {}
    FilterOperator::In => {
      if cond.value.as_ref().and_then(|v| v.as_array()).is_none() {
        return Err("operator 'in' requires an array value".to_string());
      }
    }
    _ => {
      if cond.value.is_none() {
        return Err(format!(
          "operator '{}' requires a value",
          cond.operator.as_str()
        ));
      }
    }
  }
  Ok(())
}

/// Sort direction (§3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SortDirection {
  #[default]
  Asc,
  Desc,
}

impl SortDirection {
  pub fn from_str(s: &str) -> Option<SortDirection> {
    match s.to_lowercase().as_str() {
      "asc" => Some(SortDirection::Asc),
      "desc" => Some(SortDirection::Desc),
      _ => None,
    }
  }
}

/// Single sort key: field + direction (§3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortSpec {
  pub field: String,
  pub direction: SortDirection,
}

/// Validates that sort field is in allowed sort fields.
pub fn validate_sort_field(field: &str, allowed_fields: &[&str]) -> Result<(), String> {
  if allowed_fields.is_empty() {
    return Err("no sort fields allowed for this entity".to_string());
  }
  if allowed_fields.contains(&field) {
    Ok(())
  } else {
    Err(format!(
      "invalid sort field '{}'; allowed: {}",
      field,
      allowed_fields.join(", ")
    ))
  }
}

/// Pagination defaults (§4).
pub const DEFAULT_LIMIT: u64 = 20;
pub const MAX_LIMIT: u64 = 100;

/// Offset + limit pagination (§4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffsetLimit {
  pub offset: u64,
  pub limit: u64,
}

impl Default for OffsetLimit {
  fn default() -> Self {
    Self {
      offset: 0,
      limit: DEFAULT_LIMIT,
    }
  }
}

/// Validates and normalizes offset/limit. Returns `Err` for invalid values.
pub fn validate_offset_limit(offset: u64, limit: u64) -> Result<OffsetLimit, String> {
  if limit < 1 {
    return Err("limit must be at least 1".to_string());
  }
  if limit > MAX_LIMIT {
    return Err(format!("limit must be at most {}", MAX_LIMIT));
  }
  Ok(OffsetLimit { offset, limit })
}

/// Cursor + limit pagination (§4). Cursor is opaque to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorLimit {
  pub cursor: Option<String>,
  pub limit: u64,
}

impl Default for CursorLimit {
  fn default() -> Self {
    Self {
      cursor: None,
      limit: DEFAULT_LIMIT,
    }
  }
}

/// Validates cursor+limit.
pub fn validate_cursor_limit(limit: u64) -> Result<CursorLimit, String> {
  if limit < 1 {
    return Err("limit must be at least 1".to_string());
  }
  if limit > MAX_LIMIT {
    return Err(format!("limit must be at most {}", MAX_LIMIT));
  }
  Ok(CursorLimit {
    cursor: None,
    limit,
  })
}

/// Query spec for list operations: filter conditions, sort, and pagination.
/// Scope is NOT part of the spec; it is applied from request context (§5).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListQuerySpec {
  pub filter: Vec<FilterCond>,
  pub sort: Option<SortSpec>,
  pub offset_limit: Option<OffsetLimit>,
  pub cursor_limit: Option<CursorLimit>,
}

impl ListQuerySpec {
  /// Applies default limit when no pagination given (§6).
  pub fn effective_limit(&self) -> u64 {
    if let Some(ref ol) = self.offset_limit {
      ol.limit
    } else if let Some(ref cl) = self.cursor_limit {
      cl.limit
    } else {
      DEFAULT_LIMIT
    }
  }

  pub fn effective_offset(&self) -> u64 {
    self.offset_limit.as_ref().map(|ol| ol.offset).unwrap_or(0)
  }
}
