use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// Returns the base cudup directory (~/.cudup)
pub fn cudup_home() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".cudup"))
}

/// Returns the versions directory (~/.cudup/versions)
pub fn versions_dir() -> Result<PathBuf> {
    Ok(cudup_home()?.join("versions"))
}

/// Returns the downloads directory (~/.cudup/downloads)
pub fn downloads_dir() -> Result<PathBuf> {
    Ok(cudup_home()?.join("downloads"))
}

/// Prompt user for confirmation with [y/N] style prompt
pub fn prompt_confirmation(message: &str) -> Result<bool> {
    print!("{} [y/N] ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}

/// Get list of installed CUDA versions
pub fn get_installed_versions() -> Result<Vec<String>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cudup_home() {
        let home = cudup_home().unwrap();
        assert!(home.to_string_lossy().contains(".cudup"));
    }

    #[test]
    fn test_versions_dir() {
        let versions = versions_dir().unwrap();
        assert!(versions.to_string_lossy().contains(".cudup"));
        assert!(versions.to_string_lossy().contains("versions"));
    }

    #[test]
    fn test_downloads_dir() {
        let downloads = downloads_dir().unwrap();
        assert!(downloads.to_string_lossy().contains(".cudup"));
        assert!(downloads.to_string_lossy().contains("downloads"));
    }
}
