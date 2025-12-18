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
    pub url: String,
    pub sha256: String,
    pub size: Option<u64>,
    pub relative_path: String,
}

impl DownloadTask {
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
    let response = client.get(url).send().await.context("request failed")?;

    if !response.status().is_success() {
        bail!("Download failed: HTTP {}", response.status());
    }

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).await?;
    }

    let mut file = fs::File::create(dest).await?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        if let Some(pb) = progress {
            pb.inc(chunk.len() as u64);
        }
    }

    file.flush().await?;

    Ok(())
}
