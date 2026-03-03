//! Dashboard: GET/POST/DELETE /api/dashboard/role-permissions — 401 anon, 200 read; POST/DELETE 403 viewer, 200 editor+.

use axum::http::StatusCode;

#[tokio::test]
async fn get_role_permissions_anon_401() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (status, _) =
    app::test_request(&router, "GET", "/api/dashboard/role-permissions", None, None)
      .await
      .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_role_permissions_viewer_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let (status, _) = app::test_request(
    &router,
    "GET",
    "/api/dashboard/role-permissions",
    Some(&cookie),
    None,
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn post_role_permissions_anon_401() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (status, _) = app::test_request(
    &router,
    "POST",
    "/api/dashboard/role-permissions",
    None,
    Some("{}"),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn post_role_permissions_viewer_403() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let (status, _) = app::test_request(
    &router,
    "POST",
    "/api/dashboard/role-permissions",
    Some(&cookie),
    Some("{}"),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn post_role_permissions_editor_201_or_422() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "editor@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let body = r#"{"scope":"org","role_name":"viewer","permission_key":"dashboard.users.read"}"#;
  let (status, _) = app::test_request(
    &router,
    "POST",
    "/api/dashboard/role-permissions",
    Some(&cookie),
    Some(body),
  )
  .await
  .unwrap();
  assert!(status == StatusCode::CREATED || status == StatusCode::UNPROCESSABLE_ENTITY, "POST: {}", status);
}

#[tokio::test]
async fn delete_role_permissions_viewer_403() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let body = r#"{"scope":"org","role_name":"viewer","permission_key":"dashboard"}"#;
  let (status, _) = app::test_request(
    &router,
    "DELETE",
    "/api/dashboard/role-permissions",
    Some(&cookie),
    Some(body),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn delete_role_permissions_editor_200_or_404() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "editor@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let body = r#"{"scope":"org","role_name":"viewer","permission_key":"dashboard"}"#;
  let (status, _) = app::test_request(
    &router,
    "DELETE",
    "/api/dashboard/role-permissions",
    Some(&cookie),
    Some(body),
  )
  .await
  .unwrap();
  assert!(status == StatusCode::OK || status == StatusCode::NOT_FOUND, "DELETE: {}", status);
}
