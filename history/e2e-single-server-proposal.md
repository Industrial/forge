# E2E single-server structural change (sub-second tests)

## How e2e works today

1. **bin/test-e2e** (bash):
   - Builds forge-cli, runs `forge new e2e_prebuilt`, builds the prebuilt app.
   - Runs **each** test file in its own process: `cargo test --test 001_cli`, then `--test 002_config`, … `--test 011_jobs`.

2. **Each async “server” test** (in 001–011):
   - Chooses a port via `next_e2e_port()`.
   - Writes `config/app.toml` (and sometimes `config/db.toml`) with that port.
   - **Spawns `cargo run`** in the prebuilt project (starts the app server).
   - Polls **GET /healthz** in a loop (up to 300 × 200ms = 60s) until 200.
   - Runs a few HTTP requests.
   - **Kills the server.**

So every test file that touches the server: **starts its own server**, waits for readiness (often 1–3s), does a few requests, then stops the server. That’s why e2e is slow: many server start/stop cycles and repeated readiness polling.

## Proposed change: one server for all e2e tests

- **Start the application server once** (fixed port, e.g. 30999), wait for **GET /healthz** once (short timeout, e.g. 5s).
- **Run all e2e tests** in sequence; tests **do not** spawn a server. They use a shared base URL (e.g. `E2E_BASE_URL=http://127.0.0.1:30999`) and assume the server is already up.
- **Stop the server** after all tests finish.

Effects:

- **One** server startup and one readiness wait instead of N.
- No per-test spawn/kill or polling; tests are just HTTP calls → **sub-second** per test.

## Design details

- **Config:** bin/test-e2e writes a single `config/app.toml` and `config/db.toml` (port 30999, one DB path) **before** starting the server. Tests must **not** overwrite config (they share the same process).
- **DB state:** One DB for the whole run. Tests that mutate data (auth, authz, audit) use **unique data** (e.g. unique emails per test) so they don’t conflict; tests run serially.
- **Discovery:** If `E2E_BASE_URL` (or `E2E_PORT`) is set, tests use the shared server; otherwise they can **skip** with a clear message (“run e2e via bin/test-e2e”) so `cargo test -p forge-e2e-tests` alone doesn’t hang or spawn.

## Implementation outline

1. **test/lib (forge-e2e-lib):** Add `e2e_base_url()` that returns `Option<String>` from env `E2E_BASE_URL`; add `e2e_port()` from `E2E_PORT` if needed.
2. **bin/test-e2e:** After building the app:
   - Write app.toml (port 30999) and db.toml (e.g. sqlite, auto_seed=false for predictable state).
   - Start the server in the background (`cargo run` in prebuilt dir), wait for GET /healthz (e.g. 5s timeout).
   - Export `E2E_BASE_URL=http://127.0.0.1:30999`.
   - Run all tests: `cargo test -p forge-e2e-tests` (one invocation runs all test crates) or keep the loop and run `cargo test --test 001_cli` … with env set.
   - Kill the server (trap EXIT or explicit kill).
3. **Per-test refactor:** In each async test that currently spawns a server:
   - If `e2e_base_url().is_none()`, skip the test (or return) with “run via bin/test-e2e”.
   - Else use `e2e_base_url().unwrap()` as base; **do not** spawn, write config, or poll for readiness. Just build the client and hit the base URL.

This keeps layout-only tests (sync, no server) unchanged and makes all server-hitting tests fast.
