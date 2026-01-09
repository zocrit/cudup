use crate::cache;
use crate::cuda::metadata::CudaReleaseMetadata;
use anyhow::{Context, Result};
use reqwest::Client;
use std::collections::BTreeSet;
use std::sync::LazyLock;

static VERSION_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"redistrib_(\d+\.\d+\.\d+)\.json").unwrap());

pub struct BaseDownloadUrls;

impl BaseDownloadUrls {
    pub fn cuda() -> &'static str {
        "https://developer.download.nvidia.com/compute/cuda/redist"
    }

    pub fn cudnn() -> &'static str {
        "https://developer.download.nvidia.com/compute/cudnn/redist"
    }
}

/// Fetches the list of available CUDA versions from NVIDIA's redist index
/// Uses cache if available and not expired (24h TTL)
pub async fn fetch_available_cuda_versions() -> Result<BTreeSet<String>> {
    fetch_available_cuda_versions_with_options(false).await
}

/// Fetches CUDA versions with option to force refresh
pub async fn fetch_available_cuda_versions_with_options(
    force_refresh: bool,
) -> Result<BTreeSet<String>> {
    // Try cache first
    if let Some(cached) = cache::load_cached_cuda_versions(force_refresh).await? {
        return Ok(cached);
    }

    // Fetch from network
    let client = Client::new();
    let response = client
        .get(format!("{}/", BaseDownloadUrls::cuda()))
        .send()
        .await
        .context("Failed to fetch CUDA versions index")?;

    let body = response
        .text()
        .await
        .context("Failed to read CUDA versions response")?;

    let versions = parse_available_versions(&body)?;

    // Save to cache
    cache::save_cuda_versions(&versions).await?;

    Ok(versions)
}

/// Fetches the list of available cuDNN versions from NVIDIA's redist index
/// Uses cache if available and not expired (24h TTL)
pub async fn fetch_available_cudnn_versions() -> Result<BTreeSet<String>> {
    fetch_available_cudnn_versions_with_options(false).await
}

/// Fetches cuDNN versions with option to force refresh
pub async fn fetch_available_cudnn_versions_with_options(
    force_refresh: bool,
) -> Result<BTreeSet<String>> {
    // Try cache first
    if let Some(cached) = cache::load_cached_cudnn_versions(force_refresh).await? {
        return Ok(cached);
    }

    // Fetch from network
    let client = Client::new();
    let response = client
        .get(format!("{}/", BaseDownloadUrls::cudnn()))
        .send()
        .await
        .context("Failed to fetch cuDNN versions index")?;

    let body = response
        .text()
        .await
        .context("Failed to read cuDNN versions response")?;

    let versions = parse_available_versions(&body)?;

    // Save to cache
    cache::save_cudnn_versions(&versions).await?;

    Ok(versions)
}

/// Parses version strings from the HTML directory listing
pub fn parse_available_versions(html: &str) -> Result<BTreeSet<String>> {
    let versions = VERSION_REGEX
        .captures_iter(html)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect();

    Ok(versions)
}

/// Fetches the detailed metadata JSON for a specific CUDA version
/// Uses cache if available and not expired (7 day TTL)
pub async fn fetch_cuda_version_metadata(version: &str) -> Result<CudaReleaseMetadata> {
    fetch_cuda_version_metadata_with_options(version, false).await
}

/// Fetches CUDA metadata with option to force refresh
pub async fn fetch_cuda_version_metadata_with_options(
    version: &str,
    force_refresh: bool,
) -> Result<CudaReleaseMetadata> {
    // Try cache first
    if let Some(cached) = cache::load_cached_cuda_metadata(version, force_refresh).await? {
        return Ok(cached);
    }

    // Fetch from network
    let client = Client::new();
    let url = format!("{}/redistrib_{}.json", BaseDownloadUrls::cuda(), version);

    let response = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("Failed to fetch CUDA {} metadata from {}", version, url))?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to fetch CUDA {} metadata: HTTP {}",
            version,
            response.status()
        );
    }

    let metadata: CudaReleaseMetadata = response
        .json()
        .await
        .with_context(|| format!("Failed to parse CUDA {} metadata JSON", version))?;

    // Save to cache
    cache::save_cuda_metadata(version, &metadata).await?;

    Ok(metadata)
}

/// Fetches the detailed metadata JSON for a specific cuDNN version
/// Uses cache if available and not expired (7 day TTL)
pub async fn fetch_cudnn_version_metadata(version: &str) -> Result<CudaReleaseMetadata> {
    fetch_cudnn_version_metadata_with_options(version, false).await
}

/// Fetches cuDNN metadata with option to force refresh
pub async fn fetch_cudnn_version_metadata_with_options(
    version: &str,
    force_refresh: bool,
) -> Result<CudaReleaseMetadata> {
    // Try cache first
    if let Some(cached) = cache::load_cached_cudnn_metadata(version, force_refresh).await? {
        return Ok(cached);
    }

    // Fetch from network
    let client = Client::new();
    let url = format!("{}/redistrib_{}.json", BaseDownloadUrls::cudnn(), version);

    let response = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("Failed to fetch cuDNN {} metadata from {}", version, url))?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to fetch cuDNN {} metadata: HTTP {}",
            version,
            response.status()
        );
    }

    let metadata: CudaReleaseMetadata = response
        .json()
        .await
        .with_context(|| format!("Failed to parse cuDNN {} metadata JSON", version))?;

    // Save to cache
    cache::save_cudnn_metadata(version, &metadata).await?;

    Ok(metadata)
}
