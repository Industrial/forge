//! End-to-End tests for Forge Authorization (Phase 6)

use forge::prelude::*;
use http::{header, Request, StatusCode};
use std::fs;
use tower::ServiceExt;

#[derive(Clone, Debug, ForgeAuthUser)]
#[allow(dead_code)]
struct MockUser {
  id: Uuid,
  email: String,
  password_hash: String,
  org_id: Uuid,
}

// Implement AuthzContext for MockUser so AuthSession can use it
impl AuthzContext for MockUser {
  fn requester_id(&self) -> Uuid {
    self.id
  }
  fn subject_id(&self) -> Uuid {
    self.id
  }
  fn organization_id(&self) -> Option<Uuid> {
    Some(self.org_id)
  }
}

use dashmap::DashMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
struct MockBackend {
  users: Arc<DashMap<Uuid, MockUser>>,
}

#[async_trait]
impl AuthnBackend for MockBackend {
  type User = MockUser;
  type Credentials = Uuid; // Use org_id as credentials for testing
  type Error = forge::Error;

  async fn authenticate(
    &self,
    org_id: Self::Credentials,
  ) -> Result<Option<Self::User>, Self::Error> {
    let id = Uuid::new_v4();
    let user = MockUser {
      id,
      email: format!("user@org-{}.com", org_id),
      password_hash: "hash".to_string(),
      org_id,
    };
    self.users.insert(id, user.clone());
    Ok(Some(user))
  }

  async fn get_user(
    &self,
    user_id: &forge::axum_login::UserId<Self>,
  ) -> Result<Option<Self::User>, Self::Error> {
    Ok(self.users.get(user_id).map(|u| u.clone()))
  }
}

#[tokio::test]
async fn forge_authz_ghost_mode_isolation() {
  let temp_dir = tempfile::tempdir().unwrap();
  let original_cwd = std::env::current_dir().unwrap();
  std::env::set_current_dir(temp_dir.path()).unwrap();

  fs::create_dir_all("config").unwrap();
  fs::write(
    "config/app.toml",
    r#"[app]
name = "authz_test"
environment = "test"
[server]
host = "127.0.0.1"
port = 3000
"#,
  )
  .unwrap();
  fs::write(
    "config/db.toml",
    r#"[database]
url = "sqlite::memory:"
auto_migrate = true
auto_seed = false
"#,
  )
  .unwrap();

  // We'll focus on testing the Context and Guard functionality first.
  // Testing ForgeScoped macro requires a full SeaORM environment which is
  // better tested in a generated project.

  // We'll test the logic by manually invoking what the macro should do

  let org_a = Uuid::new_v4();
  let org_b = Uuid::new_v4();
  let users = Arc::new(DashMap::new());
  let backend = MockBackend {
    users: users.clone(),
  };

  let app = App::new()
    .with_auth(move |_db| backend.clone())
    .route(
      "/login-a",
      move |mut session: AuthSession<MockBackend>| async move {
        let user = session.authenticate(org_a).await.unwrap().unwrap();
        session.login(&user).await.unwrap();
        StatusCode::OK
      },
    )
    .route(
      "/login-b",
      move |mut session: AuthSession<MockBackend>| async move {
        let user = session.authenticate(org_b).await.unwrap().unwrap();
        session.login(&user).await.unwrap();
        StatusCode::OK
      },
    )
    .route(
      "/debug-context",
      |session: AuthSession<MockBackend>| async move {
        if let Some(org_id) = session.organization_id() {
          org_id.to_string()
        } else {
          "none".to_string()
        }
      },
    )
    .route(
      "/admin-only",
      |session: AuthSession<MockBackend>| async move {
        match session.guard(Action::Manage, Role::Admin) {
          Ok(_) => (StatusCode::OK, "Success"),
          Err(_) => (StatusCode::FORBIDDEN, "Forbidden"),
        }
      },
    );

  let router = app.into_router().await;

  // 1. Login as User A
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .uri("/login-a")
        .body(axum::body::Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  let cookie_a = response.headers().get(header::SET_COOKIE).unwrap().clone();

  // 2. Verify Org A context
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .uri("/debug-context")
        .header(header::COOKIE, &cookie_a)
        .body(axum::body::Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(String::from_utf8_lossy(&body), org_a.to_string());

  // 3. Login as User B
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .uri("/login-b")
        .body(axum::body::Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  let cookie_b = response.headers().get(header::SET_COOKIE).unwrap().clone();

  // 4. Verify Org B context
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .uri("/debug-context")
        .header(header::COOKIE, &cookie_b)
        .body(axum::body::Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(String::from_utf8_lossy(&body), org_b.to_string());

  // 5. Test Guard (should pass as any authenticated user for now)
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .uri("/admin-only")
        .header(header::COOKIE, &cookie_b)
        .body(axum::body::Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);

  // 6. Test Guard fail (no cookie)
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .uri("/admin-only")
        .body(axum::body::Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::FORBIDDEN);

  std::env::set_current_dir(original_cwd).unwrap();
}
