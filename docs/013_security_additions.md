# Security Additions: CSRF, Clickjacking, CSP & OWASP-Aligned Headers

This document describes security controls Forge should add to meet high standards: **CSRF protection**, **clickjacking protection**, **Content-Security-Policy (CSP)**, and related HTTP security headers. Recommendations are aligned with **OWASP** cheat sheets, the **OWASP Secure Headers Project**, and current best practice.

## Standards & References

| Source | Use |
|--------|-----|
| [OWASP Top 10](https://owasp.org/Top10) | A01 Broken Access Control (CSRF), A03 Injection (XSS), A05 Security Misconfiguration |
| [OWASP HTTP Security Headers Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/HTTP_Headers_Cheat_Sheet.html) | Header-by-header recommendations |
| [OWASP CSRF Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html) | Token patterns, Fetch Metadata, SameSite |
| [OWASP Content Security Policy Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Content_Security_Policy_Cheat_Sheet.html) | CSP directives, strict CSP |
| [OWASP Secure Headers Project](https://owasp.org/www-project-secure-headers/) | Tooling and adoption |
| [Mozilla Observatory](https://observatory.mozilla.org/) | Header testing |

---

## 1. CSRF (Cross-Site Request Forgery) Protection

### Why

CSRF tricks a user’s browser into sending state-changing requests (POST/PUT/DELETE) to a trusted site using the user’s cookies. Without protection, the server cannot tell legitimate requests from forged ones.

**OWASP:** Use framework built-in CSRF when available; otherwise implement at least one of: synchronizer token, double-submit cookie (prefer **signed/HMAC**, session-bound), or **Fetch Metadata** with fallback.

### Options (highest standard first)

1. **Synchronizer token pattern**  
   - Server generates a unique, unpredictable token per session (or per request for maximum security; per-session is common for UX).  
   - Token is sent to the client (e.g. in HTML form as hidden field, or in JSON/header).  
   - Client sends token back in a **custom header** or form field (not in a cookie).  
   - Server validates token against session; reject if missing or invalid.  
   - **Rule:** Token must not be in cookie, URL, or logs. Use constant-time comparison.

2. **Signed double-submit cookie (recommended if stateless preferred)**  
   - HMAC(session_id + random, secret). Store HMAC+random in cookie (not HttpOnly so JS can read).  
   - Client sends same value in header or form. Server recomputes HMAC and uses constant-time compare.  
   - Must be **session-bound** (include session ID in HMAC input); naive double-submit is vulnerable to cookie injection.

3. **Fetch Metadata (Sec-Fetch-Site, etc.)**  
   - For state-changing methods (POST/PUT/PATCH/DELETE), reject requests with `Sec-Fetch-Site: cross-site`.  
   - Allow `same-origin`; treat `same-site` per threat model.  
   - **Fallback required:** If `Sec-Fetch-*` is absent (legacy clients), fall back to Origin/Referer check or token.

4. **SameSite cookie attribute**  
   - Session cookies: `SameSite=Strict` or `Lax`. Reduces CSRF from cross-site requests; **not sufficient alone** per OWASP—combine with token or Fetch Metadata.

**Critical:** XSS can defeat CSRF mitigations (e.g. steal token). CSP and input/output encoding are prerequisites.

### Forge implementation outline

- **Middleware:** For state-changing methods, require either (a) valid CSRF token in header (e.g. `X-CSRF-Token`) or (b) `Sec-Fetch-Site: same-origin` (and optionally same-site), with Origin/Referer fallback when Fetch Metadata is missing.
- **Token:** Provide a way to generate and validate tokens (e.g. HMAC double-submit or session-bound synchronizer). Expose token to the client via response body or header so SPA/form can send it back.
- **Config:** Allow disabling CSRF for selected paths (e.g. webhooks, API-only with Bearer auth).
- **Cookie:** Ensure session cookie uses `SameSite=Lax` or `Strict`, `Secure` in production, and appropriate path.

---

## 2. Clickjacking (X-Frame-Options & CSP frame-ancestors)

### Why

Clickjacking loads the app in an invisible or disguised iframe so the user thinks they’re clicking on something else. Restricting framing prevents this.

**OWASP:** Prefer CSP `frame-ancestors`; use `X-Frame-Options` for older clients. Do not allow framing by default.

### Recommendations

- **CSP `frame-ancestors`** (preferred):  
  - `frame-ancestors 'none';` — no framing.  
  - Or `frame-ancestors 'self';` — same origin only.
- **X-Frame-Options** (fallback):  
  - `X-Frame-Options: DENY` (or `SAMEORIGIN` if same-origin framing is required).

**Note:** For JSON APIs with no UI, these headers add little; still safe to send. For any response that can be loaded in a frame (HTML, error pages), they are important.

### Forge implementation outline

- **Middleware:** Add a security-headers layer that sets:
  - `X-Frame-Options: DENY` (or configurable `SAMEORIGIN`).
  - CSP including `frame-ancestors 'none';` or `'self';` (see CSP section).
- **Config:** Optional toggle and allowlist (e.g. allow a trusted admin domain).

---

## 3. Content-Security-Policy (CSP)

### Why

CSP mitigates XSS, clickjacking, and some cross-site leaks by restricting where scripts, styles, and other resources can load from and whether inline script is allowed.

**OWASP:** Prefer a **strict CSP** (nonce or hash-based). Avoid `'unsafe-inline'` and `'unsafe-eval'` for scripts when possible.

### Recommended directives (high security)

- **default-src 'none';** — deny by default; then allow only what’s needed.
- **script-src** — `'self'` and/or nonces/hashes; avoid `'unsafe-inline'`/`'unsafe-eval'`.
- **style-src 'self'** (and nonces if inline styles required).
- **img-src 'self'** (and data: / trusted CDNs if needed).
- **connect-src 'self'** (and API origins if different).
- **frame-ancestors 'none'** or **'self'** — replaces X-Frame-Options for framing.
- **form-action 'self'** — limit where forms submit.
- **base-uri 'self'** — prevent base-tag hijacking.
- **object-src 'none'** — disable plugins (Flash, etc.).

Strict CSP (e.g. [web.dev strict CSP](https://web.dev/strict-csp/)) uses **nonce** or **hash** for `script-src` and optionally `strict-dynamic` to reduce maintenance.

### API-only / JSON responses

For REST APIs that return only JSON, CSP is often unnecessary (browser doesn’t execute scripts from the response). Sending a minimal CSP or omitting it for `application/json` is acceptable; still set **X-Content-Type-Options** and other headers.

### Forge implementation outline

- **Config:** Allow app to define CSP string or structured directives (e.g. in `config/app.toml` or code).
- **Middleware:** Add `Content-Security-Policy` (and optionally `Content-Security-Policy-Report-Only` with `report-uri`/`report-to` for tuning).
- **Nonce:** If Forge ever serves HTML (templates or error pages), support injecting a per-request nonce into CSP and into `<script nonce="...">` so strict CSP is possible without `'unsafe-inline'`.

---

## 4. Full Set of OWASP-Recommended Security Headers

Implement the following for highest alignment with OWASP and modern best practice.

| Header | Recommended value | Purpose |
|--------|-------------------|--------|
| **X-Frame-Options** | `DENY` or `SAMEORIGIN` | Clickjacking (fallback) |
| **X-Content-Type-Options** | `nosniff` | Prevent MIME sniffing (e.g. script disguised as image) |
| **X-XSS-Protection** | `0` | Disable legacy XSS filter (can introduce bugs); rely on CSP |
| **Referrer-Policy** | `strict-origin-when-cross-origin` | Limit referrer leakage |
| **Strict-Transport-Security (HSTS)** | `max-age=63072000; includeSubDomains; preload` | Force HTTPS (only when TLS is correctly deployed) |
| **Content-Security-Policy** | See §3 | XSS, framing, injection |
| **Permissions-Policy** | e.g. `geolocation=(), camera=(), microphone=()` | Disable unneeded browser features |
| **Cross-Origin-Opener-Policy** | `same-origin` | Isolate browsing context (Spectre-style mitigations) |
| **Cross-Origin-Resource-Policy** | `same-site` | Limit which origins can load the resource |
| **Server** / **X-Powered-By** | Remove or generic value | Reduce fingerprinting |

**Caveats:**

- **HSTS:** Only enable when HTTPS is correctly configured; misconfiguration can lock users out. Preload only after testing.
- **COEP (`require-corp`):** Can break cross-origin resources; enable only if you control all resources or add CORP/CORS where needed.
- **CSP:** Start with report-only or a loose policy and tighten using violation reports to avoid breaking legitimate content.

---

## 5. Implementation Checklist for Forge

### Phase 1: Low-risk headers (no app logic change)

- [ ] **X-Content-Type-Options: nosniff** — all responses.
- [ ] **X-Frame-Options: DENY** (or configurable).
- [ ] **Referrer-Policy: strict-origin-when-cross-origin**.
- [ ] **Permissions-Policy** — disable unneeded features (e.g. geolocation, camera, microphone).
- [ ] **Cross-Origin-Resource-Policy: same-site** (or same-origin).
- [ ] Remove or genericize **Server** / **X-Powered-By** if present.

### Phase 2: CSP and framing in CSP

- [ ] **Content-Security-Policy** with at least `default-src 'none'; frame-ancestors 'none';` (or `'self'`) and minimal fetch directives for the app (e.g. `script-src 'self'; connect-src 'self'`). Prefer strict CSP (nonce/hash) for HTML.
- [ ] Ensure **frame-ancestors** is set so clickjacking is covered by CSP where supported.

### Phase 3: CSRF

- [ ] **SameSite** on session cookie (`Lax` or `Strict`).
- [ ] **CSRF token** (synchronizer or signed double-submit, session-bound) for state-changing requests, with validation middleware.
- [ ] **Fetch Metadata** check for state-changing methods: reject `Sec-Fetch-Site: cross-site` when token is not present; fallback to Origin/Referer when Fetch Metadata absent.
- [ ] Config to exempt paths (e.g. webhooks, token-authenticated API).

### Phase 4: HSTS and optional hardening

- [ ] **Strict-Transport-Security** when TLS is mandatory and correctly configured.
- [ ] **Cross-Origin-Opener-Policy: same-origin** if compatible with app (e.g. no cross-origin popups that need to communicate).
- [ ] **CSP report-uri** / **report-to** for monitoring and tightening.

---

## 6. Testing & Validation

- **Mozilla Observatory** — run against the app URL to score headers.
- **OWASP / SmartScanner** — dedicated tests for headers.
- **Manual:** Verify CSRF token is required for POST/PUT/DELETE and that forged cross-origin requests without token (and without SameSite bypass) are rejected.
- **CSP:** Use `Content-Security-Policy-Report-Only` and fix violations before switching to enforcing policy.

---

## 7. Status

**Not yet implemented.** Forge currently does not set CSRF protection, X-Frame-Options, CSP, or the full set of OWASP-recommended headers. This document is the design and standard reference for adding them to achieve high security standards.
