use anyhow::{bail, Result};
use std::io::{self, Write};
use std::path::PathBuf;
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

fn get_active_version_path() -> Option<PathBuf> {
    env::var("CUDA_HOME").ok().map(PathBuf::from)
}

fn is_active_version(version_path: &std::path::Path) -> bool {
    get_active_version_path()
        .map(|cuda_path| {
            match (cuda_path.canonicalize(), version_path.canonicalize()) {
                (Ok(a), Ok(b)) => a == b,
                _ => cuda_path == version_path,
            }
        })
        .unwrap_or(false)
}

fn get_installed_versions() -> Result<Vec<String>> {
    let versions_path = versions_dir()?;

    if !versions_path.exists() {
        return Ok(vec![]);
    }

    let versions: Vec<String> = fs::read_dir(&versions_path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();

    Ok(versions)
}

fn uninstall_single(version: &str, force: bool) -> Result<()> {
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

fn uninstall_all(force: bool) -> Result<()> {
    let versions_dir = versions_dir()?;
    let versions = get_installed_versions()?;

    if versions.is_empty() {
        println!("No CUDA versions installed.");
        return Ok(());
    }

    // Check if any version is active
    let active_version = versions.iter().find(|v| {
        let version_path = versions_dir.join(v);
        is_active_version(&version_path)
    });

    // If active version exists and --force not provided, error out
    if let Some(active) = active_version {
        if !force {
            bail!(
                "Cannot remove all versions - CUDA {} is currently active.\n\
                 Use --force to remove anyway, or switch versions first.",
                active
            );
        }
    }

    // Calculate total size
    let mut total_size = 0u64;
    println!("This will remove {} CUDA version(s):", versions.len());
    for version in &versions {
        let version_path = versions_dir.join(version);
        let size = dir_size(&version_path)?;
        total_size += size;

        let active_marker = if is_active_version(&version_path) {
            " (active)"
        } else {
            ""
        };
        println!("  - {}{} ({})", version, active_marker, format_size(size));
    }
    println!("\nTotal: {}", format_size(total_size));

    if active_version.is_some() {
        println!();
        println!("Warning: The active version will be removed.");
        println!("Your current shell environment will have invalid CUDA paths after removal.");
    }

    println!();

    // Ask for confirmation unless --force
    if !force {
        if !prompt_confirmation("Proceed with uninstall?")? {
            println!("Uninstall cancelled.");
            return Ok(());
        }
    }

    // Remove all versions
    for version in &versions {
        let version_path = versions_dir.join(version);
        fs::remove_dir_all(&version_path)?;
        println!("Removed CUDA {}", version);
    }

    println!("\nRemoved {} version(s)", versions.len());

    if active_version.is_some() {
        println!("\nNote: Start a new shell to clear the stale CUDA_HOME.");
    }

    Ok(())
}

pub fn uninstall(version: Option<&str>, force: bool, all: bool) -> Result<()> {
    if all {
        uninstall_all(force)
    } else if let Some(v) = version {
        uninstall_single(v, force)
    } else {
        bail!("Please specify a version or use --all")
    }
}
