# test-integration

Rust integration tests for the app. The test runner starts the API server, then runs
`cargo test -p test-integration -- --test-threads=1`.

- **Run:** `moon test-integration` or `bash bin/test-integration` from project root.
- **Location:** `tests/*.rs` in this crate.
