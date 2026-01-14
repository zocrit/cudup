use anyhow::{Context, Result, bail};
use std::path::Path;
use tokio::fs;

pub async fn extract_tarball(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    use std::process::Stdio;
    use tokio::process::Command;

    fs::create_dir_all(dest_dir).await?;

    // Determine compression type from extension
    let tar_args = if archive_path.extension().is_some_and(|ext| ext == "xz") {
        vec![
            "xf",
            archive_path.to_str().unwrap(),
            "-C",
            dest_dir.to_str().unwrap(),
            "--strip-components=1",
        ]
    } else if archive_path.extension().is_some_and(|ext| ext == "gz") {
        vec![
            "xzf",
            archive_path.to_str().unwrap(),
            "-C",
            dest_dir.to_str().unwrap(),
            "--strip-components=1",
        ]
    } else {
        vec![
            "xf",
            archive_path.to_str().unwrap(),
            "-C",
            dest_dir.to_str().unwrap(),
            "--strip-components=1",
        ]
    };

    let output = Command::new("tar")
        .args(&tar_args)
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
