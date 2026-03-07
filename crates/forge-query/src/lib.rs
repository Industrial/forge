//! Unified query specification: filter, sort, and pagination types with validation.
//! Scope is applied from request context only, not part of the spec.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

  pub fn try_parse(s: &str) -> Option<FilterOperator> {
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

impl FromStr for FilterOperator {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::try_parse(s).ok_or(())
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
  pub fn try_parse(s: &str) -> Option<SortDirection> {
    match s.to_lowercase().as_str() {
      "asc" => Some(SortDirection::Asc),
      "desc" => Some(SortDirection::Desc),
      _ => None,
    }
  }
}

impl FromStr for SortDirection {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::try_parse(s).ok_or(())
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

#[cfg(test)]
mod bdd_tests {
  use super::*;

  /// BDD-style tests focusing on behavior rather than implementation.
  /// Tests are organized by feature/behavior area with descriptive names.
  mod filter_operator_behavior {
    use super::*;

    #[test]
    fn should_convert_all_operators_to_string_representation() {
      // Given: all filter operators
      // When: converting to string
      // Then: each operator should have correct string representation
      assert_eq!(FilterOperator::Eq.as_str(), "eq");
      assert_eq!(FilterOperator::Ne.as_str(), "ne");
      assert_eq!(FilterOperator::Gt.as_str(), "gt");
      assert_eq!(FilterOperator::Gte.as_str(), "gte");
      assert_eq!(FilterOperator::Lt.as_str(), "lt");
      assert_eq!(FilterOperator::Lte.as_str(), "lte");
      assert_eq!(FilterOperator::In.as_str(), "in");
      assert_eq!(FilterOperator::Contains.as_str(), "contains");
      assert_eq!(FilterOperator::StartsWith.as_str(), "starts_with");
      assert_eq!(FilterOperator::EndsWith.as_str(), "ends_with");
      assert_eq!(FilterOperator::IsNull.as_str(), "is_null");
    }

    #[test]
    fn should_parse_valid_operator_strings() {
      // Given: valid operator strings
      // When: parsing them
      // Then: should return correct operators
      assert_eq!(FilterOperator::try_parse("eq"), Some(FilterOperator::Eq));
      assert_eq!(FilterOperator::try_parse("ne"), Some(FilterOperator::Ne));
      assert_eq!(FilterOperator::try_parse("gt"), Some(FilterOperator::Gt));
      assert_eq!(FilterOperator::try_parse("gte"), Some(FilterOperator::Gte));
      assert_eq!(FilterOperator::try_parse("lt"), Some(FilterOperator::Lt));
      assert_eq!(FilterOperator::try_parse("lte"), Some(FilterOperator::Lte));
      assert_eq!(FilterOperator::try_parse("in"), Some(FilterOperator::In));
      assert_eq!(
        FilterOperator::try_parse("contains"),
        Some(FilterOperator::Contains)
      );
      assert_eq!(
        FilterOperator::try_parse("starts_with"),
        Some(FilterOperator::StartsWith)
      );
      assert_eq!(
        FilterOperator::try_parse("ends_with"),
        Some(FilterOperator::EndsWith)
      );
      assert_eq!(
        FilterOperator::try_parse("is_null"),
        Some(FilterOperator::IsNull)
      );
    }

    #[test]
    fn should_reject_invalid_operator_strings() {
      // Given: invalid operator strings
      // When: parsing them
      // Then: should return None
      assert_eq!(FilterOperator::try_parse("invalid"), None);
      assert_eq!(FilterOperator::try_parse(""), None);
      assert_eq!(FilterOperator::try_parse("EQ"), None); // case sensitive
      assert_eq!(FilterOperator::try_parse("equals"), None);
    }

    #[test]
    fn should_parse_via_fromstr_trait() {
      // Given: valid operator strings
      // When: using FromStr trait
      // Then: should parse correctly
      assert_eq!("eq".parse::<FilterOperator>(), Ok(FilterOperator::Eq));
      assert_eq!(
        "is_null".parse::<FilterOperator>(),
        Ok(FilterOperator::IsNull)
      );
      assert_eq!("invalid".parse::<FilterOperator>(), Err(()));
    }

    #[test]
    fn should_have_roundtrip_string_conversion() {
      // Given: all filter operators
      // When: converting to string and back
      // Then: should get original operator
      for &op in FilterOperator::ALL {
        let s = op.as_str();
        let parsed = FilterOperator::try_parse(s);
        assert_eq!(parsed, Some(op), "Roundtrip failed for operator: {:?}", op);
      }
    }
  }

  mod filter_field_validation_behavior {
    use super::*;

    #[test]
    fn should_accept_valid_filter_fields() {
      // Given: allowed fields list and valid field name
      let allowed_fields = &["name", "email", "age"];

      // When: validating a field in the allowed list
      let result = validate_filter_field("name", allowed_fields);

      // Then: should succeed
      assert!(result.is_ok(), "Valid field should be accepted");
    }

    #[test]
    fn should_reject_invalid_filter_fields() {
      // Given: allowed fields list and invalid field name
      let allowed_fields = &["name", "email", "age"];

      // When: validating a field not in the allowed list
      let result = validate_filter_field("invalid_field", allowed_fields);

      // Then: should return descriptive error
      assert!(result.is_err());
      let error = result.unwrap_err();
      assert!(error.contains("invalid filter field 'invalid_field'"));
      assert!(error.contains("allowed: name, email, age"));
    }

    #[test]
    fn should_reject_empty_allowed_fields_list() {
      // Given: empty allowed fields list
      let allowed_fields = &[];

      // When: validating any field
      let result = validate_filter_field("any_field", allowed_fields);

      // Then: should return error about no fields allowed
      assert!(result.is_err());
      let error = result.unwrap_err();
      assert!(error.contains("no filter fields allowed"));
    }
  }

  mod filter_condition_validation_behavior {
    use super::*;

    #[test]
    fn should_accept_valid_filter_condition_with_value() {
      // Given: valid filter condition with value
      let cond = FilterCond {
        field: "name".to_string(),
        operator: FilterOperator::Eq,
        value: Some(serde_json::json!("test")),
      };
      let allowed_fields = &["name", "email"];

      // When: validating the condition
      let result = validate_filter_cond(&cond, allowed_fields);

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[test]
    fn should_accept_is_null_operator_without_value() {
      // Given: filter condition with IsNull operator and no value
      let cond = FilterCond {
        field: "deleted_at".to_string(),
        operator: FilterOperator::IsNull,
        value: None,
      };
      let allowed_fields = &["deleted_at"];

      // When: validating the condition
      let result = validate_filter_cond(&cond, allowed_fields);

      // Then: should succeed (IsNull doesn't require value)
      assert!(result.is_ok());
    }

    #[test]
    fn should_accept_in_operator_with_array_value() {
      // Given: filter condition with In operator and array value
      let cond = FilterCond {
        field: "status".to_string(),
        operator: FilterOperator::In,
        value: Some(serde_json::json!(["active", "pending"])),
      };
      let allowed_fields = &["status"];

      // When: validating the condition
      let result = validate_filter_cond(&cond, allowed_fields);

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[test]
    fn should_reject_in_operator_without_array_value() {
      // Given: filter condition with In operator but non-array value
      let cond = FilterCond {
        field: "status".to_string(),
        operator: FilterOperator::In,
        value: Some(serde_json::json!("active")), // single value, not array
      };
      let allowed_fields = &["status"];

      // When: validating the condition
      let result = validate_filter_cond(&cond, allowed_fields);

      // Then: should return error about array requirement
      assert!(result.is_err());
      let error = result.unwrap_err();
      assert!(error.contains("operator 'in' requires an array value"));
    }

    #[test]
    fn should_reject_operator_requiring_value_when_value_missing() {
      // Given: filter condition with operator requiring value but value is None
      let cond = FilterCond {
        field: "name".to_string(),
        operator: FilterOperator::Eq,
        value: None,
      };
      let allowed_fields = &["name"];

      // When: validating the condition
      let result = validate_filter_cond(&cond, allowed_fields);

      // Then: should return error about missing value
      assert!(result.is_err());
      let error = result.unwrap_err();
      assert!(error.contains("operator 'eq' requires a value"));
    }

    #[test]
    fn should_reject_invalid_field_in_filter_condition() {
      // Given: filter condition with invalid field
      let cond = FilterCond {
        field: "invalid_field".to_string(),
        operator: FilterOperator::Eq,
        value: Some(serde_json::json!("test")),
      };
      let allowed_fields = &["name", "email"];

      // When: validating the condition
      let result = validate_filter_cond(&cond, allowed_fields);

      // Then: should return field validation error
      assert!(result.is_err());
      let error = result.unwrap_err();
      assert!(error.contains("invalid filter field"));
    }
  }

  mod sort_direction_behavior {
    use super::*;

    #[test]
    fn should_parse_ascending_direction() {
      // Given: "asc" string (case insensitive)
      // When: parsing sort direction
      // Then: should return Asc
      assert_eq!(SortDirection::try_parse("asc"), Some(SortDirection::Asc));
      assert_eq!(SortDirection::try_parse("ASC"), Some(SortDirection::Asc));
      assert_eq!(SortDirection::try_parse("Asc"), Some(SortDirection::Asc));
    }

    #[test]
    fn should_parse_descending_direction() {
      // Given: "desc" string (case insensitive)
      // When: parsing sort direction
      // Then: should return Desc
      assert_eq!(SortDirection::try_parse("desc"), Some(SortDirection::Desc));
      assert_eq!(SortDirection::try_parse("DESC"), Some(SortDirection::Desc));
      assert_eq!(SortDirection::try_parse("Desc"), Some(SortDirection::Desc));
    }

    #[test]
    fn should_reject_invalid_direction_strings() {
      // Given: invalid direction strings
      // When: parsing them
      // Then: should return None
      assert_eq!(SortDirection::try_parse("invalid"), None);
      assert_eq!(SortDirection::try_parse(""), None);
      assert_eq!(SortDirection::try_parse("ascending"), None);
    }

    #[test]
    fn should_have_asc_as_default() {
      // Given: default SortDirection
      // When: creating default instance
      // Then: should be Asc
      let default: SortDirection = Default::default();
      assert_eq!(default, SortDirection::Asc);
    }

    #[test]
    fn should_parse_via_fromstr_trait() {
      // Given: valid direction strings
      // When: using FromStr trait
      // Then: should parse correctly
      assert_eq!("asc".parse::<SortDirection>(), Ok(SortDirection::Asc));
      assert_eq!("DESC".parse::<SortDirection>(), Ok(SortDirection::Desc));
      assert_eq!("invalid".parse::<SortDirection>(), Err(()));
    }
  }

  mod sort_field_validation_behavior {
    use super::*;

    #[test]
    fn should_accept_valid_sort_fields() {
      // Given: allowed sort fields and valid field name
      let allowed_fields = &["name", "created_at", "updated_at"];

      // When: validating a field in the allowed list
      let result = validate_sort_field("name", allowed_fields);

      // Then: should succeed
      assert!(result.is_ok());
    }

    #[test]
    fn should_reject_invalid_sort_fields() {
      // Given: allowed sort fields and invalid field name
      let allowed_fields = &["name", "created_at"];

      // When: validating a field not in the allowed list
      let result = validate_sort_field("invalid_field", allowed_fields);

      // Then: should return descriptive error
      assert!(result.is_err());
      let error = result.unwrap_err();
      assert!(error.contains("invalid sort field 'invalid_field'"));
      assert!(error.contains("allowed: name, created_at"));
    }

    #[test]
    fn should_reject_empty_allowed_sort_fields_list() {
      // Given: empty allowed sort fields list
      let allowed_fields = &[];

      // When: validating any field
      let result = validate_sort_field("any_field", allowed_fields);

      // Then: should return error about no fields allowed
      assert!(result.is_err());
      let error = result.unwrap_err();
      assert!(error.contains("no sort fields allowed"));
    }
  }

  mod offset_limit_validation_behavior {
    use super::*;

    #[test]
    fn should_accept_valid_offset_and_limit() {
      // Given: valid offset and limit values
      // When: validating them
      let result = validate_offset_limit(10, 50);

      // Then: should succeed and return OffsetLimit
      assert!(result.is_ok());
      let ol = result.unwrap();
      assert_eq!(ol.offset, 10);
      assert_eq!(ol.limit, 50);
    }

    #[test]
    fn should_reject_limit_less_than_one() {
      // Given: limit value of 0
      // When: validating offset and limit
      let result = validate_offset_limit(0, 0);

      // Then: should return error about minimum limit
      assert!(result.is_err());
      let error = result.unwrap_err();
      assert!(error.contains("limit must be at least 1"));
    }

    #[test]
    fn should_reject_limit_greater_than_max() {
      // Given: limit value exceeding MAX_LIMIT
      // When: validating offset and limit
      let result = validate_offset_limit(0, MAX_LIMIT + 1);

      // Then: should return error about maximum limit
      assert!(result.is_err());
      let error = result.unwrap_err();
      assert!(error.contains(&format!("limit must be at most {}", MAX_LIMIT)));
    }

    #[test]
    fn should_accept_limit_at_maximum() {
      // Given: limit value at MAX_LIMIT
      // When: validating offset and limit
      let result = validate_offset_limit(0, MAX_LIMIT);

      // Then: should succeed
      assert!(result.is_ok());
      let ol = result.unwrap();
      assert_eq!(ol.limit, MAX_LIMIT);
    }

    #[test]
    fn should_have_default_offset_limit() {
      // Given: default OffsetLimit
      // When: creating default instance
      // Then: should have offset 0 and limit DEFAULT_LIMIT
      let default: OffsetLimit = Default::default();
      assert_eq!(default.offset, 0);
      assert_eq!(default.limit, DEFAULT_LIMIT);
    }
  }

  mod cursor_limit_validation_behavior {
    use super::*;

    #[test]
    fn should_accept_valid_limit() {
      // Given: valid limit value
      // When: validating cursor limit
      let result = validate_cursor_limit(50);

      // Then: should succeed and return CursorLimit with None cursor
      assert!(result.is_ok());
      let cl = result.unwrap();
      assert_eq!(cl.cursor, None);
      assert_eq!(cl.limit, 50);
    }

    #[test]
    fn should_reject_limit_less_than_one() {
      // Given: limit value of 0
      // When: validating cursor limit
      let result = validate_cursor_limit(0);

      // Then: should return error about minimum limit
      assert!(result.is_err());
      let error = result.unwrap_err();
      assert!(error.contains("limit must be at least 1"));
    }

    #[test]
    fn should_reject_limit_greater_than_max() {
      // Given: limit value exceeding MAX_LIMIT
      // When: validating cursor limit
      let result = validate_cursor_limit(MAX_LIMIT + 1);

      // Then: should return error about maximum limit
      assert!(result.is_err());
      let error = result.unwrap_err();
      assert!(error.contains(&format!("limit must be at most {}", MAX_LIMIT)));
    }

    #[test]
    fn should_accept_limit_at_maximum() {
      // Given: limit value at MAX_LIMIT
      // When: validating cursor limit
      let result = validate_cursor_limit(MAX_LIMIT);

      // Then: should succeed
      assert!(result.is_ok());
      let cl = result.unwrap();
      assert_eq!(cl.limit, MAX_LIMIT);
    }

    #[test]
    fn should_have_default_cursor_limit() {
      // Given: default CursorLimit
      // When: creating default instance
      // Then: should have None cursor and limit DEFAULT_LIMIT
      let default: CursorLimit = Default::default();
      assert_eq!(default.cursor, None);
      assert_eq!(default.limit, DEFAULT_LIMIT);
    }
  }

  mod list_query_spec_behavior {
    use super::*;

    #[test]
    fn should_return_offset_limit_when_offset_limit_present() {
      // Given: ListQuerySpec with offset_limit set
      let spec = ListQuerySpec {
        offset_limit: Some(OffsetLimit {
          offset: 10,
          limit: 30,
        }),
        cursor_limit: None,
        ..Default::default()
      };

      // When: getting effective limit
      let limit = spec.effective_limit();

      // Then: should return offset_limit's limit
      assert_eq!(limit, 30);
    }

    #[test]
    fn should_return_cursor_limit_when_only_cursor_limit_present() {
      // Given: ListQuerySpec with only cursor_limit set
      let spec = ListQuerySpec {
        offset_limit: None,
        cursor_limit: Some(CursorLimit {
          cursor: None,
          limit: 50,
        }),
        ..Default::default()
      };

      // When: getting effective limit
      let limit = spec.effective_limit();

      // Then: should return cursor_limit's limit
      assert_eq!(limit, 50);
    }

    #[test]
    fn should_prefer_offset_limit_over_cursor_limit() {
      // Given: ListQuerySpec with both offset_limit and cursor_limit set
      let spec = ListQuerySpec {
        offset_limit: Some(OffsetLimit {
          offset: 5,
          limit: 25,
        }),
        cursor_limit: Some(CursorLimit {
          cursor: None,
          limit: 75,
        }),
        ..Default::default()
      };

      // When: getting effective limit
      let limit = spec.effective_limit();

      // Then: should prefer offset_limit's limit
      assert_eq!(limit, 25);
    }

    #[test]
    fn should_return_default_limit_when_no_pagination_present() {
      // Given: ListQuerySpec with no pagination
      let spec = ListQuerySpec {
        offset_limit: None,
        cursor_limit: None,
        ..Default::default()
      };

      // When: getting effective limit
      let limit = spec.effective_limit();

      // Then: should return DEFAULT_LIMIT
      assert_eq!(limit, DEFAULT_LIMIT);
    }

    #[test]
    fn should_return_offset_from_offset_limit_when_present() {
      // Given: ListQuerySpec with offset_limit set
      let spec = ListQuerySpec {
        offset_limit: Some(OffsetLimit {
          offset: 15,
          limit: 30,
        }),
        ..Default::default()
      };

      // When: getting effective offset
      let offset = spec.effective_offset();

      // Then: should return offset_limit's offset
      assert_eq!(offset, 15);
    }

    #[test]
    fn should_return_zero_offset_when_no_offset_limit_present() {
      // Given: ListQuerySpec without offset_limit
      let spec = ListQuerySpec {
        offset_limit: None,
        cursor_limit: Some(CursorLimit {
          cursor: None,
          limit: 50,
        }),
        ..Default::default()
      };

      // When: getting effective offset
      let offset = spec.effective_offset();

      // Then: should return 0
      assert_eq!(offset, 0);
    }

    #[test]
    fn should_have_default_list_query_spec() {
      // Given: default ListQuerySpec
      // When: creating default instance
      // Then: should have empty filter, no sort, no pagination
      let default: ListQuerySpec = Default::default();
      assert!(default.filter.is_empty());
      assert!(default.sort.is_none());
      assert!(default.offset_limit.is_none());
      assert!(default.cursor_limit.is_none());
    }
  }
}
