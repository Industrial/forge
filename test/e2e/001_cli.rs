//! E2E tests for Forge CLI using the prebuilt project from `bin/test-e2e`.
//!
//! **Pattern**: Use the single prebuilt project at `.tmp/e2e_prebuilt` (created and built by
//! `bin/test-e2e`). Health endpoints are tested only in 008_health.

use forge_e2e_lib::cli;

#[test]
fn e2e_prebuilt_project_layout() {
  let project_root = cli::prebuilt_project_root();
  assert!(
    project_root.exists(),
    "prebuilt project not found at {} — run bin/test-e2e first",
    project_root.display()
  );
  cli::assert_project_layout(&project_root);
}
