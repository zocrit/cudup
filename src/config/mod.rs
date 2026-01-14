use anyhow::{Context, Result};
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
