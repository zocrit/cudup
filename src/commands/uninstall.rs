use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::{env, fs};

use crate::config::{get_installed_versions, prompt_confirmation, versions_dir};
use crate::fetch::format_size;

fn dir_size(path: &Path) -> Result<u64> {
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

fn is_active_version(version_path: &Path) -> bool {
    get_active_version_path().is_some_and(|cuda_path| {
        match (cuda_path.canonicalize(), version_path.canonicalize()) {
            (Ok(a), Ok(b)) => a == b,
            _ => cuda_path == version_path,
        }
    })
}

fn uninstall_single(version: &str, force: bool) -> Result<()> {
    let versions_dir = versions_dir()?;
    let version_path = versions_dir.join(version);

    if !version_path.exists() {
        bail!("CUDA {} is not installed", version);
    }

    let is_active = is_active_version(&version_path);

    let size = dir_size(&version_path)?;

    println!("This will remove CUDA {}:", version);
    println!("  - {} ({})", version_path.display(), format_size(size));

    if is_active {
        println!();
        println!("Warning: This version is currently active (CUDA_HOME points to it).");
        println!("Your current shell environment will have invalid CUDA paths after removal.");
    }

    println!();

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

    match fs::remove_dir_all(&version_path) {
        Ok(()) => {
            println!();
            println!("Removed CUDA {}", version);
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            bail!("CUDA {} was already removed by another process", version);
        }
        Err(e) => {
            return Err(e).context(format!("Failed to remove CUDA {}", version));
        }
    }

    if is_active {
        println!();
        println!("Note: Run 'cudup use <version>' to activate a different version,");
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

    let active_version = versions.iter().find(|v| {
        let version_path = versions_dir.join(v);
        is_active_version(&version_path)
    });

    if let Some(active) = active_version
        && !force
    {
        bail!(
            "Cannot remove all versions - CUDA {} is currently active.\n\
                 Use --force to remove anyway, or switch versions first.",
            active
        );
    }

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
    println!();
    println!("Total: {}", format_size(total_size));

    if active_version.is_some() {
        println!();
        println!("Warning: The active version will be removed.");
        println!("Your current shell environment will have invalid CUDA paths after removal.");
    }

    println!();

    if !force && !prompt_confirmation("Proceed with uninstall?")? {
        println!("Uninstall cancelled.");
        return Ok(());
    }

    let mut removed_count = 0;
    for version in &versions {
        let version_path = versions_dir.join(version);
        match fs::remove_dir_all(&version_path) {
            Ok(()) => {
                println!("Removed CUDA {}", version);
                removed_count += 1;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!("CUDA {} was already removed", version);
            }
            Err(e) => {
                return Err(e).context(format!("Failed to remove CUDA {}", version));
            }
        }
    }

    println!();
    println!("Removed {} version(s)", removed_count);

    if active_version.is_some() {
        println!();
        println!("Note: Start a new shell to clear the stale CUDA_HOME.");
    }

    Ok(())
}

pub fn uninstall(version: Option<&str>, force: bool, all: bool) -> Result<()> {
    match (all, version) {
        (true, _) => uninstall_all(force),
        (false, Some(v)) => uninstall_single(v, force),
        (false, None) => bail!("Please specify a version or use --all"),
    }
}
