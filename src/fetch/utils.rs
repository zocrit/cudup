use anyhow::{Result, bail};
use std::path::PathBuf;

use crate::config;

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
