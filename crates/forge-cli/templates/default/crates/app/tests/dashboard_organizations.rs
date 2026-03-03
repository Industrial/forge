//! Dashboard: GET/POST/PATCH/DELETE /api/dashboard/organizations — 401 anon, 403 non-global-admin, 200 global_admin.

use axum::http::StatusCode;

#[tokio::test]
async fn get_organizations_anon_401() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (status, _) = app::test_request(&router, "GET", "/api/dashboard/organizations", None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_organizations_viewer_403() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let (status, _) = app::test_request(
    &router,
    "GET",
    "/api/dashboard/organizations",
    Some(&cookie),
    None,
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn get_organizations_global_admin_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "admin@admin.com", app::SEED_PASSWORD)
    .await
    .expect("login");
  let (status, body) = app::test_request(
    &router,
    "GET",
    "/api/dashboard/organizations",
    Some(&cookie),
    None,
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::OK);
  let json: app::serde_json::Value = app::serde_json::from_slice(&body).unwrap();
  let orgs = json["organizations"].as_array().unwrap();
  assert!(!orgs.is_empty());
}

#[tokio::test]
async fn post_organizations_viewer_403() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let body = r#"{"name":"New Org","slug":"new-org"}"#;
  let (status, _) = app::test_request(
    &router,
    "POST",
    "/api/dashboard/organizations",
    Some(&cookie),
    Some(body),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn post_organizations_global_admin_201() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "admin@admin.com", app::SEED_PASSWORD)
    .await
    .expect("login");
  let body = r#"{"name":"Test Org","slug":"test-org-12345"}"#;
  let (status, _) = app::test_request(
    &router,
    "POST",
    "/api/dashboard/organizations",
    Some(&cookie),
    Some(body),
  )
  .await
  .unwrap();
  assert_eq!(status, StatusCode::CREATED);
}
