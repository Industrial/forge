//! End-to-End tests for Forge Authorization (Phase 6)
//!
//! **Covered:** Org isolation (context), Shallow Gate (`guard`) for all roles (Owner, Admin, Editor, Viewer),
//! role hierarchy (Owner > Admin > Editor > Viewer), unauthenticated → Forbidden, and user with no role → Forbidden.
//!
//! **Not covered here:** Deep Scope (ForgeScoped / `.scoped(&auth)`) and ForgePolicy (ReBAC) require a full
//! SeaORM + generated-app environment and are better exercised via a generated project or integration tests.

use async_trait::async_trait;
use axum_login::{AuthSession, AuthnBackend};
use forge::authz::{Action, AuthSessionGuardExt, AuthzContext, Role};
use forge::{App, Error as ForgeError, ForgeAuthUser};
use http::{Request, StatusCode, header};
use std::fs;
use tower::ServiceExt;
use uuid::Uuid;

#[derive(Clone, Debug, ForgeAuthUser)]
#[allow(dead_code)]
struct MockUser {
  id: Uuid,
  email: String,
  password_hash: String,
  org_id: Uuid,
  role: Option<Role>,
}

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
  fn role(&self) -> Option<Role> {
    self.role.clone()
  }
}

use dashmap::DashMap;
use std::sync::Arc;

/// Credentials for the mock backend: (org_id, role). Used to create users with different roles.
#[derive(Clone, Debug)]
struct MockCreds(pub Uuid, pub Role);

#[derive(Clone, Debug)]
struct MockBackend {
  users: Arc<DashMap<Uuid, MockUser>>,
}

#[async_trait]
impl AuthnBackend for MockBackend {
  type User = MockUser;
  type Credentials = MockCreds;
  type Error = ForgeError;

  async fn authenticate(
    &self,
    creds: Self::Credentials,
  ) -> Result<Option<Self::User>, Self::Error> {
    let id = Uuid::new_v4();
    let user = MockUser {
      id,
      email: format!("user@org-{}.com", creds.0),
      password_hash: "hash".to_string(),
      org_id: creds.0,
      role: Some(creds.1),
    };
    self.users.insert(id, user.clone());
    Ok(Some(user))
  }

  async fn get_user(
    &self,
    user_id: &forge::axum_login::UserId<Self>,
  ) -> Result<Option<Self::User>, Self::Error> {
    Ok(self.users.get(user_id).map(|r| (*r).clone()))
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
        let user = session
          .authenticate(MockCreds(org_a, Role::Admin))
          .await
          .unwrap()
          .unwrap();
        session.login(&user).await.unwrap();
        StatusCode::OK
      },
    )
    .route(
      "/login-b",
      move |mut session: AuthSession<MockBackend>| async move {
        let user = session
          .authenticate(MockCreds(org_b, Role::Admin))
          .await
          .unwrap()
          .unwrap();
        session.login(&user).await.unwrap();
        StatusCode::OK
      },
    )
    .route(
      "/login-owner",
      move |mut session: AuthSession<MockBackend>| async move {
        let user = session
          .authenticate(MockCreds(org_a, Role::Owner))
          .await
          .unwrap()
          .unwrap();
        session.login(&user).await.unwrap();
        StatusCode::OK
      },
    )
    .route(
      "/login-editor",
      move |mut session: AuthSession<MockBackend>| async move {
        let user = session
          .authenticate(MockCreds(org_a, Role::Editor))
          .await
          .unwrap()
          .unwrap();
        session.login(&user).await.unwrap();
        StatusCode::OK
      },
    )
    .route(
      "/login-viewer",
      move |mut session: AuthSession<MockBackend>| async move {
        let user = session
          .authenticate(MockCreds(org_a, Role::Viewer))
          .await
          .unwrap()
          .unwrap();
        session.login(&user).await.unwrap();
        StatusCode::OK
      },
    )
    .route(
      "/login-no-role",
      move |mut session: AuthSession<MockBackend>| async move {
        let id = Uuid::new_v4();
        let user = MockUser {
          id,
          email: "norole@test.com".to_string(),
          password_hash: "hash".to_string(),
          org_id: org_a,
          role: None,
        };
        users.insert(id, user.clone());
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
      "/owner-only",
      |session: AuthSession<MockBackend>| async move {
        match session.guard(Action::Manage, Role::Owner) {
          Ok(_) => (StatusCode::OK, "Success"),
          Err(_) => (StatusCode::FORBIDDEN, "Forbidden"),
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
    )
    .route(
      "/viewer-only",
      |session: AuthSession<MockBackend>| async move {
        match session.guard(Action::Read, Role::Viewer) {
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

  // 5. Test Guard (authenticated user with Role::Admin can access admin-only)
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

  // 6. Test Guard fail (no cookie / unauthenticated)
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

  // --- Role hierarchy and insufficient-role tests ---

  // 7. Login as Owner (org_a)
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .uri("/login-owner")
        .body(axum::body::Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  let cookie_owner = response.headers().get(header::SET_COOKIE).unwrap().clone();

  // 8. Owner can access owner-only, admin-only, viewer-only (hierarchy)
  for uri in ["/owner-only", "/admin-only", "/viewer-only"] {
    let response = router
      .clone()
      .oneshot(
        Request::builder()
          .uri(uri)
          .header(header::COOKIE, &cookie_owner)
          .body(axum::body::Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(
      response.status(),
      StatusCode::OK,
      "Owner should access {}",
      uri
    );
  }

  // 9. Login as Viewer (org_a)
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .uri("/login-viewer")
        .body(axum::body::Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  let cookie_viewer = response.headers().get(header::SET_COOKIE).unwrap().clone();

  // 10. Viewer can access only viewer-only; forbidden for owner-only and admin-only
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .uri("/viewer-only")
        .header(header::COOKIE, &cookie_viewer)
        .body(axum::body::Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);

  for uri in ["/owner-only", "/admin-only"] {
    let response = router
      .clone()
      .oneshot(
        Request::builder()
          .uri(uri)
          .header(header::COOKIE, &cookie_viewer)
          .body(axum::body::Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(
      response.status(),
      StatusCode::FORBIDDEN,
      "Viewer should be forbidden for {}",
      uri
    );
  }

  // 11. Login as Editor (org_a) – can access viewer-only (Editor > Viewer), forbidden for owner-only and admin-only
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .uri("/login-editor")
        .body(axum::body::Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  let cookie_editor = response.headers().get(header::SET_COOKIE).unwrap().clone();

  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .uri("/viewer-only")
        .header(header::COOKIE, &cookie_editor)
        .body(axum::body::Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Editor satisfies Viewer in hierarchy"
  );

  for uri in ["/owner-only", "/admin-only"] {
    let response = router
      .clone()
      .oneshot(
        Request::builder()
          .uri(uri)
          .header(header::COOKIE, &cookie_editor)
          .body(axum::body::Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(
      response.status(),
      StatusCode::FORBIDDEN,
      "Editor must not satisfy {}",
      uri
    );
  }

  // 12. User with no role (role None) gets Forbidden on any guarded route
  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .uri("/login-no-role")
        .body(axum::body::Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  let cookie_norole = response.headers().get(header::SET_COOKIE).unwrap().clone();

  for uri in ["/owner-only", "/admin-only", "/viewer-only"] {
    let response = router
      .clone()
      .oneshot(
        Request::builder()
          .uri(uri)
          .header(header::COOKIE, &cookie_norole)
          .body(axum::body::Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(
      response.status(),
      StatusCode::FORBIDDEN,
      "User with no role should be forbidden for {}",
      uri
    );
  }

  std::env::set_current_dir(original_cwd).unwrap();
}
