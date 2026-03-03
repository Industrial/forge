//! Dashboard: GET /api/dashboard/permissions — 401 anon, 200 any auth.

use axum::http::StatusCode;

#[tokio::test]
async fn get_permissions_anon_401() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let (status, _) = app::test_request(&router, "GET", "/api/dashboard/permissions", None, None)
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_permissions_viewer_default_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let (status, _) =
    app::test_request(&router, "GET", "/api/dashboard/permissions", Some(&cookie), None)
      .await
      .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_permissions_editor_default_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "editor@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let (status, _) =
    app::test_request(&router, "GET", "/api/dashboard/permissions", Some(&cookie), None)
      .await
      .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_permissions_admin_default_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "orgadmin@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let (status, _) =
    app::test_request(&router, "GET", "/api/dashboard/permissions", Some(&cookie), None)
      .await
      .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_permissions_owner_default_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "owner@default.org", app::SEED_PASSWORD)
    .await
    .expect("login");
  let (status, _) =
    app::test_request(&router, "GET", "/api/dashboard/permissions", Some(&cookie), None)
      .await
      .unwrap();
  assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_permissions_global_admin_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");
  let cookie = app::login_as_seed_user(&router, "admin@admin.com", app::SEED_PASSWORD)
    .await
    .expect("login");
  let (status, _) =
    app::test_request(&router, "GET", "/api/dashboard/permissions", Some(&cookie), None)
      .await
      .unwrap();
  assert_eq!(status, StatusCode::OK);
}
