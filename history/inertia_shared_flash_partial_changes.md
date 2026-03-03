# Inertia: Shared Data, Flash Messages, Partial Reloads — Files and Changes

**Historical note:** This plan assumed session-based auth with `tower-sessions` and `Session` extractors. Forge has since moved to **token-based auth** (Bearer token + scope headers). Session and tower-sessions have been removed; flash/shared data would be implemented differently (e.g. one-time tokens in redirects or client state).

---

This document lists every file changed to implement **(1) shared data**, **(2) flash messages**, and **(3) partial reloads** in the default Forge Inertia template.

---

## 1. **New file: `crates/forge-cli/templates/default/crates/app/src/handlers/inertia_shared.rs`**

New module that:

- **Shared data:** Builds a JSON object `{ auth: { user }, flash: { message, error }, appName }` and merges it with every page’s props.
- **Flash:** Reads `flash_message` and `flash_error` from the session, includes them in shared props, and removes them after read (one-time).
- **Partial reloads:** Defines `MergedProps` (shared + page) and implements `axum_inertia::Props` so that when the client sends `X-Inertia-Partial-Data` (e.g. `only: ['message']`), the response includes only those top-level keys from the merged props.

Key pieces:

- `FLASH_MESSAGE`, `FLASH_ERROR` — session key constants (exported for auth handlers).
- `shared_props(session, user)` — async; builds shared JSON and clears flash keys.
- `MergedProps` — holds `shared` and `page` `Value`s; `Props::serialize(partial)` merges them and, if `partial` is `Some`, filters the merged object to the requested keys.
- `render_with_shared(i, session, user, component, page_props)` — builds shared props, wraps in `MergedProps`, and calls `i.render(component, merged)`.

---

## 2. **`crates/forge-cli/templates/default/crates/app/Cargo.toml`**

- **Add dependency:** `tower-sessions = "0.14"`  
  So the app can use the `Session` extractor and `session.insert` / `session.get` / `session.remove` for flash.

---

## 3. **`crates/forge-cli/templates/default/crates/app/src/handlers/mod.rs`**

- **Add:** `pub mod inertia_shared;`

---

## 4. **`crates/forge-cli/templates/default/crates/app/src/handlers/inertia.rs`**

- **Imports:** `tower_sessions::Session`, `crate::handlers::inertia_shared`.
- **Every Inertia handler:** Add `Session` and `OptionalRequireAuth<Backend>` (so shared data can include `auth.user`). Replace `i.render(...)` with:

  `inertia_shared::render_with_shared(i, session, maybe_user.as_ref(), "Pages/...", json!({ ... })).await`

- **Handlers updated:** `home`, `login_page`, `register_page`, `dashboard`, `ws_demo_page`.
- **dashboard:** Still returns `Redirect::to("/login")` when unauthenticated; when authenticated, uses `render_with_shared` so the dashboard page gets shared data (and partial reloads) as well.

---

## 5. **`crates/forge-cli/templates/default/crates/app/src/handlers/auth.rs`**

- **Imports:** `tower_sessions::Session`, `crate::handlers::inertia_shared`.
- **`register`:** Add extractor `session: Session`. After successful registration, before redirect:

  `session.insert(inertia_shared::FLASH_MESSAGE, "Thanks for registering. Please log in.").await.ok();`

- **`login`:** Add extractor `session: Session`. After successful login, before redirect:

  `session.insert(inertia_shared::FLASH_MESSAGE, "Welcome back!").await.ok();`

So the next full or Inertia request (e.g. login page or dashboard) will see `flash.message` in shared props once.

---

## 6. **`crates/forge-cli/templates/default/crates/app/src/main.rs`**

- **No code change** for shared/flash/partial.
- **Note:** If your app merges an API router (with `State<DbConnection>`) and an Inertia router (with `State<AppState>`), Axum’s `merge` requires a single state type. So you may need to use one state type (e.g. `AppState`) for both and implement `FromRef<AppState> for DbConnection` (already in `state.rs`), then:

  - Build: `let app_state = AppState { db: db_conn, inertia };`
  - Use: `let api_router = router.with_state(app_state.clone());` and `let inertia_router = ... .with_state(app_state);`

  If the forge API only allows `router.with_state(db_conn)`, you would need a forge change (e.g. router generic over state) to use a single `AppState` for both routers.

---

## 7. **New file: `crates/forge-cli/templates/default/frontend/src/components/Layout.tsx`**

- **Purpose:** Use shared data and flash on the client.
- **Uses** `usePage().props` typed as `{ auth, flash, appName }`:
  - Renders “Logged in as {email}” when `auth.user` is set.
  - Renders a success alert when `flash.message` is set.
  - Renders an error alert when `flash.error` is set.
- **Exports** a `Layout` wrapper that takes `children` and wraps the page content with header + flash + article.

---

## 8. **`crates/forge-cli/templates/default/frontend/src/pages/Dashboard.tsx`**

- **Wrap content in** `<Layout>` so the dashboard shows shared auth and flash (e.g. “Welcome back!” after login).
- **Imports:** `Layout` from `../components/Layout`.

---

## Summary table

| File | Change |
|------|--------|
| **New** `handlers/inertia_shared.rs` | Shared props (auth, flash, appName), flash read/clear, `MergedProps` + partial filtering, `render_with_shared`. |
| **New** `frontend/src/components/Layout.tsx` | Layout that shows `auth.user` and `flash.message` / `flash.error` via `usePage().props`. |
| `app/Cargo.toml` | Add `tower-sessions = "0.14"`. |
| `handlers/mod.rs` | Add `pub mod inertia_shared`. |
| `handlers/inertia.rs` | Add `Session` + `OptionalRequireAuth` to all handlers; use `render_with_shared` instead of `i.render`. |
| `handlers/auth.rs` | Add `Session` to `register` and `login`; set `FLASH_MESSAGE` after success. |
| `frontend/src/pages/Dashboard.tsx` | Wrap page in `<Layout>`. |
| `main.rs` | No change (optional: unify state so merge uses one state type). |

---

## Flow

1. **Shared data:** Every Inertia response is built via `render_with_shared`, which merges shared `{ auth, flash, appName }` with page props. So every page receives `auth.user`, `flash`, and `appName` in `usePage().props`.
2. **Flash:** After register/login, the handler sets `session.insert(FLASH_MESSAGE, ...)`. On the next request, `shared_props` reads those keys, includes them in shared props, and removes them from the session so they are one-time.
3. **Partial reloads:** When the client sends a partial reload (e.g. `only: ['message']`), `MergedProps::serialize(partial)` returns only the requested top-level keys from the merged (shared + page) object, so the client can refresh a subset of props.
