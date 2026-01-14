use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

// ============================================================================
// CLI Basic Tests
// ============================================================================

#[test]
fn test_help_output() {
    let mut cmd = Command::cargo_bin("cudup").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("cudup"))
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn test_version_output() {
    let mut cmd = Command::cargo_bin("cudup").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("cudup"));
}

#[test]
fn test_install_without_version_fails() {
    let mut cmd = Command::cargo_bin("cudup").unwrap();
    cmd.arg("install")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"))
        .stderr(predicate::str::contains("VERSION"));
}

#[test]
fn test_list_help() {
    let mut cmd = Command::cargo_bin("cudup").unwrap();
    cmd.args(["list", "--help"]).assert().success();
}

// ============================================================================
// Use Command Tests
// ============================================================================

#[test]
fn test_use_without_version_fails() {
    let mut cmd = Command::cargo_bin("cudup").unwrap();
    cmd.arg("use")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"))
        .stderr(predicate::str::contains("VERSION"));
}

#[test]
fn test_use_noninstalled_version_fails() {
    let mut cmd = Command::cargo_bin("cudup").unwrap();
    cmd.args(["use", "99.99.99"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not installed"))
        .stderr(predicate::str::contains("cudup install"));
}

#[test]
fn test_use_installed_version_outputs_exports() {
    // Create a fake installed version
    let home = dirs::home_dir().expect("Could not get home directory");
    let version_dir = home.join(".cudup/versions/99.88.77");

    // Skip if we can't create the test directory
    if fs::create_dir_all(&version_dir).is_err() {
        return;
    }

    let mut cmd = Command::cargo_bin("cudup").unwrap();
    let result = cmd.args(["use", "99.88.77"]).assert().success();

    // Verify output contains expected export commands
    result
        .stdout(predicate::str::contains("export CUDA_HOME="))
        .stdout(predicate::str::contains("99.88.77"))
        .stdout(predicate::str::contains("export PATH="))
        .stdout(predicate::str::contains("export LD_LIBRARY_PATH="));

    // Clean up
    let _ = fs::remove_dir_all(&version_dir);
}

// ============================================================================
// Install Command Tests (without actual downloads)
// ============================================================================

#[test]
fn test_install_invalid_version_format() {
    // This test verifies the command handles invalid input gracefully
    // The actual network call would fail, but we're testing the CLI behavior
    let mut cmd = Command::cargo_bin("cudup").unwrap();
    cmd.args(["install", "not-a-version"])
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .failure();
}

// ============================================================================
// Subcommand Help Tests
// ============================================================================

#[test]
fn test_install_help() {
    let mut cmd = Command::cargo_bin("cudup").unwrap();
    cmd.args(["install", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("install"));
}

#[test]
fn test_use_help() {
    let mut cmd = Command::cargo_bin("cudup").unwrap();
    cmd.args(["use", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("use"));
}

#[test]
fn test_unknown_subcommand() {
    let mut cmd = Command::cargo_bin("cudup").unwrap();
    cmd.arg("unknown-command").assert().failure();
}
