use anyhow::Result;
use std::env;
use std::path::PathBuf;
use std::process::Command;

use crate::config::{cudup_home, get_installed_versions};

struct CheckResult {
    name: String,
    status: CheckStatus,
    detail: Option<String>,
}

enum CheckStatus {
    Ok,
    Warning,
    Error,
}

impl CheckResult {
    #[must_use]
    fn ok(name: impl Into<String>, detail: Option<impl Into<String>>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Ok,
            detail: detail.map(Into::into),
        }
    }

    #[must_use]
    fn warning(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Warning,
            detail: Some(detail.into()),
        }
    }

    #[must_use]
    fn error(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Error,
            detail: Some(detail.into()),
        }
    }

    fn print(&self) {
        let symbol = match self.status {
            CheckStatus::Ok => "✓",
            CheckStatus::Warning => "!",
            CheckStatus::Error => "✗",
        };

        print!("[{}] {}", symbol, self.name);
        if let Some(detail) = &self.detail {
            print!(": {}", detail);
        }
        println!();
    }
}

fn check_cudup_home() -> CheckResult {
    match cudup_home() {
        Ok(path) if path.exists() => {
            CheckResult::ok("cudup directory", Some(&path.display().to_string()))
        }
        Ok(path) => CheckResult::warning(
            "cudup directory",
            &format!(
                "{} does not exist (run 'cudup manage setup')",
                path.display()
            ),
        ),
        Err(e) => CheckResult::error("cudup directory", &e.to_string()),
    }
}

fn check_shell_integration() -> CheckResult {
    let env_path = match cudup_home() {
        Ok(home) => home.join("env"),
        Err(e) => return CheckResult::error("shell integration", &e.to_string()),
    };

    if env_path.exists() {
        CheckResult::ok("shell integration", Some("env file exists"))
    } else {
        CheckResult::warning(
            "shell integration",
            "not configured (run 'cudup manage setup')",
        )
    }
}

fn check_installed_versions() -> CheckResult {
    let versions = match get_installed_versions() {
        Ok(v) => v,
        Err(e) => return CheckResult::error("installed versions", &e.to_string()),
    };

    if versions.is_empty() {
        CheckResult::ok("installed versions", Some("none"))
    } else {
        CheckResult::ok(
            "installed versions",
            Some(&format!("{} ({})", versions.len(), versions.join(", "))),
        )
    }
}

fn check_active_version() -> CheckResult {
    match env::var("CUDA_HOME") {
        Ok(cuda_home) => {
            let path = PathBuf::from(&cuda_home);
            if path.exists() {
                // Try to extract version from path
                let version = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                CheckResult::ok("active version", Some(version))
            } else {
                CheckResult::error(
                    "active version",
                    &format!("CUDA_HOME={} does not exist", cuda_home),
                )
            }
        }
        Err(_) => CheckResult::warning("active version", "CUDA_HOME not set"),
    }
}

fn check_nvcc() -> CheckResult {
    match Command::new("nvcc").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Extract version from output like "Cuda compilation tools, release 12.4, V12.4.99"
            let version = stdout
                .lines()
                .find(|line| line.contains("release"))
                .and_then(|line| {
                    line.split("release")
                        .nth(1)
                        .and_then(|s| s.split(',').next())
                        .map(|s| s.trim())
                })
                .unwrap_or("found");
            CheckResult::ok("nvcc", Some(version))
        }
        Ok(_) => CheckResult::warning("nvcc", "not working"),
        Err(_) => CheckResult::warning("nvcc", "not found in PATH"),
    }
}

fn check_nvidia_driver() -> CheckResult {
    match Command::new("nvidia-smi")
        .arg("--query-gpu=driver_version")
        .arg("--format=csv,noheader")
        .output()
    {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout)
                .trim()
                .lines()
                .next()
                .unwrap_or("found")
                .to_string();
            CheckResult::ok("nvidia driver", Some(&format!("v{}", version)))
        }
        Ok(_) => CheckResult::error("nvidia driver", "nvidia-smi failed"),
        Err(_) => CheckResult::warning("nvidia driver", "nvidia-smi not found"),
    }
}

fn check_gpu() -> CheckResult {
    match Command::new("nvidia-smi")
        .arg("--query-gpu=name")
        .arg("--format=csv,noheader")
        .output()
    {
        Ok(output) if output.status.success() => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let gpus: Vec<&str> = output_str.trim().lines().collect();
            let gpu_info = if gpus.len() == 1 {
                gpus[0].to_string()
            } else {
                format!("{} GPUs", gpus.len())
            };
            CheckResult::ok("gpu", Some(&gpu_info))
        }
        Ok(_) => CheckResult::warning("gpu", "could not detect"),
        Err(_) => CheckResult::warning("gpu", "nvidia-smi not available"),
    }
}

pub fn check() -> Result<()> {
    println!("cudup check\n");

    let checks = vec![
        check_cudup_home(),
        check_shell_integration(),
        check_installed_versions(),
        check_active_version(),
        check_nvcc(),
        check_nvidia_driver(),
        check_gpu(),
    ];

    for result in &checks {
        result.print();
    }

    let (errors, warnings) = checks.iter().fold((0, 0), |(e, w), c| match c.status {
        CheckStatus::Error => (e + 1, w),
        CheckStatus::Warning => (e, w + 1),
        CheckStatus::Ok => (e, w),
    });

    println!();
    if errors > 0 {
        println!("{} error(s), {} warning(s)", errors, warnings);
    } else if warnings > 0 {
        println!("No errors, {} warning(s)", warnings);
    } else {
        println!("All checks passed!");
    }

    Ok(())
}
