//! Integration test: login as seed user and GET profile (template auth).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::util::ServiceExt;

#[tokio::test]
async fn post_login_then_get_profile_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");

  let cookie = app::login_as_seed_user(&router, "viewer@default.org", app::SEED_PASSWORD)
    .await
    .expect("login as viewer@default.org");

  let req = Request::builder()
    .uri("/api/auth/profile")
    .header("cookie", cookie)
    .body(Body::empty())
    .unwrap();
  let res = router.oneshot(req).await.unwrap();
  assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn get_profile_without_cookie_401() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");

  let req = Request::builder()
    .uri("/api/auth/profile")
    .body(Body::empty())
    .unwrap();
  let res = router.oneshot(req).await.unwrap();
  assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn post_login_invalid_credentials_401() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");

  let body = r#"{"email":"viewer@default.org","password":"wrongpassword"}"#;
  let (status, _) = app::test_request(&router, "POST", "/api/auth/login", None, Some(body))
    .await
    .unwrap();
  assert_eq!(status, StatusCode::UNAUTHORIZED);
}
