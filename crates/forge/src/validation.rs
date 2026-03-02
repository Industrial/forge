//! Request validation with [validator](https://crates.io/crates/validator) and [axum-valid](https://crates.io/crates/axum-valid).
//!
//! Derive `Validate` on request structs and use `Valid<Json<T>>` or `Valid<Query<T>>` in handlers.
//! Invalid requests receive 422 Unprocessable Entity with a structured error body.

pub use axum_valid::Valid;
pub use validator::Validate;

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, Validate)]
  struct TestRequest {
    #[validate(length(min = 1, message = "name required"))]
    name: String,
  }

  #[test]
  fn validate_accepts_valid_request() {
    let r = TestRequest {
      name: "hello".to_string(),
    };
    assert!(r.validate().is_ok());
  }

  #[test]
  fn validate_rejects_invalid_request() {
    let r = TestRequest {
      name: "".to_string(),
    };
    let res = r.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err.field_errors().contains_key("name"));
  }
}
