# 010_validation: Request Validation

## Overview

Forge integrates **request validation** using [validator](https://crates.io/crates/validator) (Keats) and [axum-valid](https://crates.io/crates/axum-valid), so handlers receive typed, validated extractors and invalid requests get a consistent **422 Unprocessable Entity** response with error details.

## Goals

- **One standard**: Use **validator** as the single validation crate; axum-valid provides Axum extractors that wrap `Json`, `Query`, etc. and run validation.
- **No extra boilerplate**: Derive `Validate` on request structs; use `Valid<Json<T>>` or `Valid<Query<T>>` in handlers.
- **Stable error shape**: Document the JSON error response format (e.g. field path, message, code) for frontends and API consumers.

## Architecture

### Flow

1. Request body or query is deserialized into `T`.
2. axum-valid runs `T::validate()` (validator trait).
3. If valid, handler receives `Valid<Json<T>>` (or `Valid<Query<T>>`); if invalid, axum-valid returns 422 with a list of errors.

### Error Response

Example JSON:

```json
{
  "errors": [
    { "field": "email", "message": "invalid email" },
    { "field": "password", "message": "length must be at least 8" }
  ]
}
```

Exact shape depends on axum-valid/validator; Forge documents the chosen format and re-exports the necessary types.

### Re-exports

- `forge::validation` (or similar) re-exports:
  - `validator::Validate` (derive macro)
  - `axum_valid::Valid`
  - Common validator attributes (`email`, `length`, etc.) for documentation.

## Configuration

None required. Validation is opt-in per handler by using `Valid<Json<T>>` or `Valid<Query<T>>`.

## Usage

**Define a validated request struct:**

```rust
use forge::validation::{Valid, Validate};
use serde::Deserialize;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}
```

**Use in handler:**

```rust
async fn create_user(
    Valid(Json(body)): Valid<Json<CreateUserRequest>>,
    State(db): State<DatabaseConnection>,
) -> Result<StatusCode, Error> {
    // body is guaranteed valid
    // ...
}
```

Invalid requests never reach the handler; they receive 422 and the error list.

## Dependencies

- **validator** (with `derive`): validation trait and rules.
- **axum-valid** (with `validator` feature): extractors `Valid<Json<T>>`, `Valid<Query<T>>`, etc.

## Success Criteria

1. Handlers can use `Valid<Json<T>>` and `Valid<Query<T>>` when `T` implements `Validate`.
2. Invalid input returns 422 with a structured error body.
3. Forge docs and re-exports document the recommended pattern and error format.
4. No second validation crate (e.g. garde) in the default stack; validator only.

## Future Extensions

- Custom validators (e.g. `unique` for email in DB).
- i18n of error messages.
- Consistent error type in `forge::error` for validation failures.
