//! Integration test: health endpoints (forge layer).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::util::ServiceExt;

#[tokio::test]
async fn get_healthz_livez_readyz_return_200() {
  let (router, _guard) = app::build_router_for_test()
    .await
    .expect("build_router_for_test");

  for path in ["/healthz", "/livez", "/readyz"] {
    let req = Request::builder().uri(path).body(Body::empty()).unwrap();
    let res = router.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK, "GET {} should return 200", path);
  }
}
