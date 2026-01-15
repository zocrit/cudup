use anyhow::{bail, Result};
use std::io::{self, Write};
use std::{env, fs};

use crate::config::versions_dir;

fn prompt_confirmation(message: &str) -> Result<bool> {
    print!("{} [y/N] ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}

fn format_size(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    }
}

fn dir_size(path: &std::path::Path) -> Result<u64> {
    let mut size = 0;
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                size += dir_size(&path)?;
            } else {
                size += entry.metadata()?.len();
            }
        }
    }
    Ok(size)
}

fn is_active_version(version_path: &std::path::Path) -> bool {
    env::var("CUDA_HOME")
        .ok()
        .map(|cuda_home| {
            let cuda_path = std::path::PathBuf::from(cuda_home);
            // Canonicalize both paths for comparison, fall back to direct comparison
            match (cuda_path.canonicalize(), version_path.canonicalize()) {
                (Ok(a), Ok(b)) => a == b,
                _ => cuda_path == version_path,
            }
        })
        .unwrap_or(false)
}

pub fn uninstall(version: &str, force: bool) -> Result<()> {
    let versions_dir = versions_dir()?;
    let version_path = versions_dir.join(version);

    // Check if version is installed
    if !version_path.exists() {
        bail!("CUDA {} is not installed", version);
    }

    // Check if it's the active version
    let is_active = is_active_version(&version_path);

    // Calculate size
    let size = dir_size(&version_path)?;

    println!("This will remove CUDA {}:", version);
    println!("  - {} ({})", version_path.display(), format_size(size));

    if is_active {
        println!();
        println!("Warning: This version is currently active (CUDA_HOME points to it).");
        println!("Your current shell environment will have invalid CUDA paths after removal.");
    }

    println!();

    // Ask for confirmation unless --force
    if !force {
        let prompt = if is_active {
            "Remove active version anyway?"
        } else {
            "Proceed with uninstall?"
        };

        if !prompt_confirmation(prompt)? {
            println!("Uninstall cancelled.");
            return Ok(());
        }
    }

    // Remove the directory
    fs::remove_dir_all(&version_path)?;

    println!("\nRemoved CUDA {}", version);

    if is_active {
        println!("\nNote: Run 'cudup use <version>' to activate a different version,");
        println!("or start a new shell to clear the stale CUDA_HOME.");
    }

    Ok(())
}
