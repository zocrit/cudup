use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Creates a Command for the cudup binary.
fn cudup_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("cudup"))
}

/// Asserts that a subcommand fails when VERSION argument is missing.
fn assert_missing_version_error(subcommand: &str) {
    cudup_cmd()
        .arg(subcommand)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("required").and(predicate::str::contains("VERSION")),
        );
}

/// Asserts that a subcommand's help output is valid.
fn assert_subcommand_help(subcommand: &str) {
    cudup_cmd()
        .args([subcommand, "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(subcommand));
}

// ============================================================================
// CLI Basic Tests
// ============================================================================

#[test]
fn test_help_output() {
    cudup_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("cudup"))
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn test_version_output() {
    cudup_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("cudup"));
}

#[test]
fn test_unknown_subcommand() {
    cudup_cmd().arg("unknown-command").assert().failure();
}

// ============================================================================
// Subcommand Help Tests
// ============================================================================

#[test]
fn test_list_help() {
    assert_subcommand_help("list");
}

#[test]
fn test_install_help() {
    assert_subcommand_help("install");
}

#[test]
fn test_use_help() {
    assert_subcommand_help("use");
}

// ============================================================================
// Missing Argument Tests
// ============================================================================

#[test]
fn test_install_without_version_fails() {
    assert_missing_version_error("install");
}

#[test]
fn test_use_without_version_fails() {
    assert_missing_version_error("use");
}

// ============================================================================
// Use Command Tests
// ============================================================================

#[test]
fn test_use_noninstalled_version_fails() {
    cudup_cmd()
        .args(["use", "99.99.99"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not installed"))
        .stderr(predicate::str::contains("cudup install"));
}

#[test]
fn test_use_installed_version_outputs_exports() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let version_dir = temp_dir.path().join("versions/99.88.77");
    fs::create_dir_all(&version_dir).expect("Failed to create version dir");

    cudup_cmd()
        .env("CUDUP_HOME", temp_dir.path())
        .args(["use", "99.88.77"])
        .assert()
        .success()
        .stdout(predicate::str::contains("export CUDA_HOME="))
        .stdout(predicate::str::contains("99.88.77"))
        .stdout(predicate::str::contains("export PATH="))
        .stdout(predicate::str::contains("export LD_LIBRARY_PATH="));
}

// ============================================================================
// Install Command Tests (without actual downloads)
// ============================================================================

#[test]
fn test_install_invalid_version_format() {
    cudup_cmd()
        .args(["install", "not-a-version"])
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .failure();
}
