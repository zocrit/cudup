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
fn test_install_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "install"])
        .output()
        .expect("Failed to execute cudup install");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("install"));
}
