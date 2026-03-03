//! Auth: POST /api/auth/register — public success and validation 4xx.

use axum::http::StatusCode;

#[tokio::test]
async fn post_register_valid_201() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");

  let body = r#"{"email":"newuser@example.com","password":"password123"}"#;
  let (status, _) = app::test_request(
    &router,
    "POST",
    "/api/auth/register",
    None,
    Some(body),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn post_register_invalid_email_422() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");

  let body = r#"{"email":"not-an-email","password":"password123"}"#;
  let (status, _) = app::test_request(
    &router,
    "POST",
    "/api/auth/register",
    None,
    Some(body),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn post_register_short_password_422() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");

  let body = r#"{"email":"u@example.com","password":"short"}"#;
  let (status, _) = app::test_request(
    &router,
    "POST",
    "/api/auth/register",
    None,
    Some(body),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}
