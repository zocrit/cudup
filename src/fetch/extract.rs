use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Stdio;
use tokio::fs;
use tokio::process::Command;

pub async fn extract_tarball(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    fs::create_dir_all(dest_dir).await?;

    // Safely convert paths to strings
    let archive_str = archive_path
        .to_str()
        .context("Archive path contains invalid UTF-8")?;
    let dest_str = dest_dir
        .to_str()
        .context("Destination path contains invalid UTF-8")?;

    // Determine tar flag based on compression type
    let tar_flag = match archive_path.extension().and_then(|e| e.to_str()) {
        Some("gz") => "xzf",
        Some("xz") | _ => "xf",
    };

    let output = Command::new("tar")
        .args([tar_flag, archive_str, "-C", dest_str, "--strip-components=1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to run tar command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to extract {}: {}", archive_path.display(), stderr);
    }

    Ok(())
}
