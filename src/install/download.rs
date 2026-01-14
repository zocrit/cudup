use anyhow::{Context, Result, bail};
use reqwest::Client;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone)]
pub struct DownloadTask {
    pub package_name: String,
    pub version: String,
    pub url: String,
    pub sha256: String,
    pub size: u64,
    pub relative_path: String,
}

pub async fn download_file(client: &Client, url: &str, dest: &Path) -> Result<()> {
    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Failed to start download from {}", url))?;

    if !response.status().is_success() {
        bail!("Download failed: HTTP {}", response.status());
    }

    // Ensure parent directory exists
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).await?;
    }

    let bytes = response
        .bytes()
        .await
        .with_context(|| format!("Failed to download {}", url))?;

    let mut file = fs::File::create(dest).await?;
    file.write_all(&bytes).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_task_struct() {
        let task = DownloadTask {
            package_name: "test_pkg".to_string(),
            version: "1.0.0".to_string(),
            url: "https://example.com/test.tar.xz".to_string(),
            sha256: "abc123".to_string(),
            size: 12345,
            relative_path: "test/test.tar.xz".to_string(),
        };

        assert_eq!(task.package_name, "test_pkg");
        assert_eq!(task.size, 12345);
    }
}
