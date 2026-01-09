use std::process::Command;

#[test]
fn test_help_output() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute cudup --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cudup"));
    assert!(stdout.contains("Usage"));
}

#[test]
fn test_install_without_version_fails() {
    let output = Command::new("cargo")
        .args(["run", "--", "install"])
        .output()
        .expect("Failed to execute cudup install");

    // Should fail when no version is provided
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("specify a CUDA version"));
}
