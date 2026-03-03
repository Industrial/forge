//! Core types for the Forge framework: error and validation.
//!
//! Minimal surface with no dependency on HTTP, database, or auth so that
//! other Forge crates can depend on forge-core without pulling in the full stack.

pub mod error;
pub mod validation;

pub use error::Error;
pub use validation::{Valid, Validate};
