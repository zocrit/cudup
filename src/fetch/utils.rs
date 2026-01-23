use anyhow::{Result, bail};
use std::path::PathBuf;

use crate::config;

/// Returns the NVIDIA redistributable platform string for the current system.
///
/// Supported platforms:
/// - `linux-x86_64` - Linux on x86_64
/// - `linux-sbsa` - Linux on ARM64 (Server Base System Architecture)
///
/// Note: `linux-aarch64` (Jetson/embedded) is not supported.
/// Use NVIDIA JetPack SDK for Jetson devices.
pub fn target_platform() -> Result<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Ok("linux-x86_64"),
        ("linux", "aarch64") => Ok("linux-sbsa"),
        (os, arch) => bail!(
            "Unsupported platform: {}-{}. \
             cudup supports linux-x86_64 and linux-sbsa (ARM64 server).",
            os,
            arch
        ),
    }
}

pub fn version_install_dir(cuda_version: &str) -> Result<PathBuf> {
    Ok(config::versions_dir()?.join(cuda_version))
}

#[must_use]
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn test_format_size_kilobytes() {
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(2048), "2.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
    }

    #[test]
    fn test_format_size_megabytes() {
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 100), "100.00 MB");
        assert_eq!(format_size(1024 * 1024 + 512 * 1024), "1.50 MB");
    }

    #[test]
    fn test_format_size_gigabytes() {
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_size(1024 * 1024 * 1024 * 5), "5.00 GB");
    }

    #[test]
    fn test_version_install_dir() {
        let dir = version_install_dir("12.4.1").unwrap();
        assert!(dir.to_string_lossy().contains("12.4.1"));
        assert!(dir.to_string_lossy().contains(".cudup"));
        assert!(dir.to_string_lossy().contains("versions"));
    }

    #[test]
    fn test_target_platform() {
        let result = target_platform();

        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => {
                assert_eq!(result.unwrap(), "linux-x86_64");
            }
            ("linux", "aarch64") => {
                assert_eq!(result.unwrap(), "linux-sbsa");
            }
            (os, arch) => {
                // On unsupported platforms (like macOS), it should return an error
                let err = result.unwrap_err();
                assert!(
                    err.to_string().contains("Unsupported platform"),
                    "Expected 'Unsupported platform' error for {}-{}, got: {}",
                    os,
                    arch,
                    err
                );
            }
        }
    }
}
