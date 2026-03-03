//! Request validation (re-exported from [forge_core](forge_core)).
//!
//! Derive `Validate` on request structs and use `Valid<Json<T>>` or `Valid<Query<T>>` in handlers.
//! Invalid requests receive 422 Unprocessable Entity with a structured error body.

pub use forge_core::{Valid, Validate};
