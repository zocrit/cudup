use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

pub fn cudup_home() -> Result<PathBuf> {
    if let Ok(custom_home) = std::env::var("CUDUP_HOME") {
        return Ok(PathBuf::from(custom_home));
    }
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".cudup"))
}

pub fn versions_dir() -> Result<PathBuf> {
    Ok(cudup_home()?.join("versions"))
}

pub fn downloads_dir() -> Result<PathBuf> {
    Ok(cudup_home()?.join("downloads"))
}

pub fn prompt_confirmation(message: &str) -> Result<bool> {
    print!("{} [y/N] ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}

pub fn get_installed_versions() -> Result<Vec<String>> {
    let versions_path = versions_dir()?;

    if !versions_path.exists() {
        return Ok(vec![]);
    }

    Ok(fs::read_dir(versions_path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect())
}
