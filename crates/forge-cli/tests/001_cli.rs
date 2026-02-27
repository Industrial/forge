//! 001_cli: first test — invoke the Forge CLI and observe/expect the result.
//!
//! The directory `tests/001_cli/` (at repo root) represents the Forge application
//! for this test. Tests run by invoking the `forge` binary and asserting on
//! exit code, stdout, and stderr.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn forge_version_prints_version() {
    let mut cmd = cargo_bin_cmd!("forge");
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("forge "))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}
