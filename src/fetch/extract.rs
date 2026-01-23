use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Stdio;
use tokio::fs;
use tokio::process::Command;

pub async fn extract_tarball(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    fs::create_dir_all(dest_dir).await?;

    let output = Command::new("tar")
        .arg("xf")
        .arg(archive_path)
        .arg("-C")
        .arg(dest_dir)
        .arg("--strip-components=1")
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
