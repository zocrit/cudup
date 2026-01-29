use anyhow::{Context, Result, bail};
use futures::StreamExt;
use indicatif::ProgressBar;
use reqwest::Client;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone)]
pub struct DownloadTask {
    pub package_name: String,
    #[allow(dead_code)]
    pub version: String,
    pub url: String,
    pub sha256: String,
    pub size: Option<u64>,
    pub relative_path: String,
}

impl DownloadTask {
    /// Returns the archive filename from the relative path
    #[must_use]
    pub fn archive_name(&self) -> &str {
        self.relative_path
            .split('/')
            .next_back()
            .filter(|s| !s.is_empty())
            .unwrap_or("archive.tar.xz")
    }
}

pub async fn download_file(
    client: &Client,
    url: &str,
    dest: &Path,
    progress: Option<&ProgressBar>,
) -> Result<()> {
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

    let mut file = fs::File::create(dest).await?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.with_context(|| format!("Failed to download chunk from {}", url))?;
        file.write_all(&chunk).await?;
        if let Some(pb) = progress {
            pb.inc(chunk.len() as u64);
        }
    }

    file.flush().await?;

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
            size: Some(12345),
            relative_path: "test/path/test.tar.xz".to_string(),
        };

        assert_eq!(task.package_name, "test_pkg");
        assert_eq!(task.size, Some(12345));
    }

    #[test]
    fn test_archive_name_extracts_filename() {
        let task = DownloadTask {
            package_name: "cuda_cccl".to_string(),
            version: "12.4.127".to_string(),
            url: "https://example.com/archive.tar.xz".to_string(),
            sha256: "abc123".to_string(),
            size: Some(1000),
            relative_path: "cuda_cccl/linux-x86_64/cuda_cccl-linux-x86_64-12.4.127-archive.tar.xz"
                .to_string(),
        };

        assert_eq!(
            task.archive_name(),
            "cuda_cccl-linux-x86_64-12.4.127-archive.tar.xz"
        );
    }

    #[test]
    fn test_archive_name_simple_path() {
        let task = DownloadTask {
            package_name: "test".to_string(),
            version: "1.0.0".to_string(),
            url: "https://example.com/test.tar.xz".to_string(),
            sha256: "abc123".to_string(),
            size: Some(1000),
            relative_path: "simple.tar.xz".to_string(),
        };

        assert_eq!(task.archive_name(), "simple.tar.xz");
    }

    #[test]
    fn test_archive_name_empty_path() {
        let task = DownloadTask {
            package_name: "test".to_string(),
            version: "1.0.0".to_string(),
            url: "https://example.com/test.tar.xz".to_string(),
            sha256: "abc123".to_string(),
            size: Some(1000),
            relative_path: "".to_string(),
        };

        assert_eq!(task.archive_name(), "archive.tar.xz");
    }
}
