//! Generic REST model trait and error type for entities served by a single generic handler.
//! Implement [RestModel] to get list/get/create/update/delete via `/api/entities/{model_id}`.

mod model_error;
mod rest_model;

pub use model_error::ModelError;
pub use rest_model::{REST_ACTIONS, RestModel};
