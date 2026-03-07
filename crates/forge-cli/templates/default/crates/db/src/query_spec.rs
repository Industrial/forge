//! Re-export unified query specification from forge-query.

pub use forge_query::*;

#[cfg(test)]
mod bdd_tests {
  use super::*;

  mod re_export_behavior {
    use super::*;

    #[test]
    fn should_re_export_query_spec_types_from_forge_query() {
      // Given: query_spec module
      // When: using re-exported types
      // Then: types should be accessible
      // This is verified by successful compilation
      // Types like ListQuerySpec, FilterCond, SortSpec should be available
    }

    #[test]
    fn should_have_list_query_spec_type() {
      // Given: query_spec module
      // When: checking ListQuerySpec type
      // Then: should be available (verified by compilation)
      let _spec: ListQuerySpec = ListQuerySpec {
        filters: vec![],
        sort: vec![],
        limit: None,
        offset: None,
      };
    }
  }
}
