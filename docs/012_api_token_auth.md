# API Token / Token Auth

Forge currently supports **session-based authentication** only (cookies via `tower-sessions` and `axum-login`). This document describes **API token authentication**: Bearer tokens or API keys for headless clients, CLIs, mobile apps, and machine-to-machine access.

## Objectives

1. **Dual auth modes**: Allow the same app to accept either session cookies (browser) or a Bearer token / API key (programmatic).
2. **Token lifecycle**: Create, list, revoke tokens; optional expiry and scopes.
3. **Secure storage**: Store token hashes (never plaintext); use constant-time comparison where applicable.
4. **Unified identity**: Token auth resolves to the same `User` (and optional org/role) as session auth so authorization and audit stay consistent.

## Use cases

| Client type        | Typical auth        | Example                          |
|--------------------|---------------------|----------------------------------|
| Browser (SPA / SSR)| Session (cookie)    | Already supported via `with_auth` |
| CLI / script       | Bearer token or API key | `Authorization: Bearer <token>`  |
| Mobile app         | Bearer token        | OAuth2 access token or app-specific token |
| Third-party API    | API key (header or query) | `X-API-Key: <key>` or `?api_key=<key>` |
| Webhooks / workers | API key or service token | Fixed or per-tenant tokens       |

## Architecture (proposed)

### 1. Token storage

- **Table**: e.g. `api_tokens` or `user_tokens`:
  - `id`: UUID (PK)
  - `user_id`: UUID (FK → user)
  - `token_hash`: String (hash of the secret; e.g. SHA-256 or Argon2)
  - `name`: Optional label (e.g. "CLI", "Production script")
  - `last_used_at`: Optional timestamp
  - `expires_at`: Optional; NULL = never expires
  - `scopes`: Optional JSON or comma-separated scopes (e.g. `read`, `write`)
  - `created_at`, `updated_at`
- **Secret**: Generated once at creation (e.g. 32-byte random), shown to the user once; only the hash is stored.

### 2. Token presentation

- **Bearer** (recommended): `Authorization: Bearer <token>`.
- **API key header**: `X-API-Key: <token>` or custom header from config.
- **Query param**: `?access_token=<token>` (discouraged for GET; avoid in logs).

### 3. Middleware / extractor

- Run **after** session auth: if the request has no valid session, check for a Bearer token or API key.
- Look up token by hash, validate expiry (and optional scopes), load `User` (and org/role if applicable).
- Inject the same auth context (`axum_login`-style or Forge’s `AuthzContext`) so handlers don’t care whether the user came from a cookie or a token.
- Rate limiting and audit can treat token-authenticated requests like session-authenticated ones (e.g. per user_id).

### 4. Token creation and revocation

- **Create**: Authenticated route (e.g. `POST /api/auth/tokens`) with optional name, expiry, scopes; return the secret once in the response body.
- **List**: `GET /api/auth/tokens` (current user’s tokens, mask secret).
- **Revoke**: `DELETE /api/auth/tokens/:id` (and optionally revoke-all).

## Integration with existing Forge auth

- **`forge::auth`**: Add helpers for hashing and verifying token secrets (similar in spirit to password hashing; verification must be constant-time).
- **`App::with_auth`**: Extend or add a companion (e.g. `with_token_auth`) so the same `AuthnBackend` (or a wrapper) can resolve users from both session and token.
- **Authorization**: No change: once the requester is identified (user_id, org, role), `forge-authz` and audit behave as today.
- **Sessions**: Token auth does not create a session; it only authenticates the request. Optional: allow “create session from token” for browser flows that start with a token (e.g. magic link).

## Security considerations

- **HTTPS only** for token transmission in production.
- **Short-lived tokens** and refresh flows if you need OAuth2-style refresh (out of scope for a minimal API token feature).
- **Scopes**: Optional; restrict token capabilities (e.g. read-only) without separate users.
- **Audit**: Log token creation, use (e.g. last_used_at), and revocation in the same audit pipeline as auth/authz events.

## Success criteria (when implemented)

- Requests with `Authorization: Bearer <valid_token>` are authenticated as the owning user and pass the same authz guards as session auth.
- Token secrets are stored only as hashes; verification is constant-time.
- Optional expiry and scopes are enforced.
- E2E test: create token via session-authenticated endpoint, then call a protected route with the token and receive 200 with correct identity.

## Status

**Implemented.** Forge supports dual auth: session (cookie) and API token (Bearer). Generated apps include `api_tokens` migration, token create/list/revoke routes, and `with_token_auth(db::token_lookup)`. E2E test: `test/e2e/012_api_tokens.rs`.
