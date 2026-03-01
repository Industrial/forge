//! Request validation with [validator](https://crates.io/crates/validator) and [axum-valid](https://crates.io/crates/axum-valid).
//!
//! Derive `Validate` on request structs and use `Valid<Json<T>>` or `Valid<Query<T>>` in handlers.
//! Invalid requests receive 422 Unprocessable Entity with a structured error body.

pub use axum_valid::Valid;
pub use validator::Validate;
