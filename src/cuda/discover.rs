use crate::cuda::metadata::CudaReleaseMetadata;
use anyhow::{Context, Result};
use reqwest::Client;
use std::collections::BTreeSet;
use std::sync::LazyLock;
use std::time::Duration;

static VERSION_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"redistrib_(\d+\.\d+\.\d+)\.json").expect("invalid version regex pattern")
});

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client")
});

pub const CUDA_BASE_URL: &str = "https://developer.download.nvidia.com/compute/cuda/redist";
pub const CUDNN_BASE_URL: &str = "https://developer.download.nvidia.com/compute/cudnn/redist";

async fn fetch_available_versions(base_url: &str, product: &str) -> Result<BTreeSet<String>> {
    let response = HTTP_CLIENT
        .get(format!("{}/", base_url))
        .send()
        .await
        .with_context(|| format!("Failed to fetch {} versions index", product))?;

    let body = response
        .text()
        .await
        .with_context(|| format!("Failed to read {} versions response", product))?;

    Ok(parse_available_versions(&body))
}

async fn fetch_version_metadata(
    base_url: &str,
    product: &str,
    version: &str,
) -> Result<CudaReleaseMetadata> {
    let url = format!("{}/redistrib_{}.json", base_url, version);

    let response = HTTP_CLIENT.get(&url).send().await.with_context(|| {
        format!(
            "Failed to fetch {} {} metadata from {}",
            product, version, url
        )
    })?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to fetch {} {} metadata: HTTP {}",
            product,
            version,
            response.status()
        );
    }

    response
        .json()
        .await
        .with_context(|| format!("Failed to parse {} {} metadata JSON", product, version))
}

pub async fn fetch_available_cuda_versions() -> Result<BTreeSet<String>> {
    fetch_available_versions(CUDA_BASE_URL, "CUDA").await
}

pub async fn fetch_available_cudnn_versions() -> Result<BTreeSet<String>> {
    fetch_available_versions(CUDNN_BASE_URL, "cuDNN").await
}

pub fn parse_available_versions(html: &str) -> BTreeSet<String> {
    VERSION_REGEX
        .captures_iter(html)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

pub async fn fetch_cuda_version_metadata(version: &str) -> Result<CudaReleaseMetadata> {
    fetch_version_metadata(CUDA_BASE_URL, "CUDA", version).await
}

/// Finds the newest cuDNN version compatible with a specific CUDA version
///
/// Uses early exit optimization - iterates from newest to oldest and returns
/// on first match. This is faster than fetching all versions when you only
/// need the latest compatible one.
pub async fn find_newest_compatible_cudnn(cuda_version: &str) -> Result<Option<String>> {
    let cuda_major = cuda_version
        .split('.')
        .next()
        .context("Invalid CUDA version format")?;

    let cuda_major_str = cuda_major.to_string();
    let all_cudnn_versions = fetch_available_cudnn_versions().await?;

    for cudnn_version in all_cudnn_versions.iter().rev() {
        let metadata = match fetch_cudnn_version_metadata(cudnn_version).await {
            Ok(m) => m,
            Err(_) => continue,
        };

        let is_compatible = metadata
            .get_package("cudnn")
            .and_then(|pkg| pkg.cuda_variant.as_ref())
            .is_some_and(|variants| variants.contains(&cuda_major_str));

        if is_compatible {
            return Ok(Some(cudnn_version.clone()));
        }
    }

    Ok(None)
}

pub async fn fetch_cudnn_version_metadata(version: &str) -> Result<CudaReleaseMetadata> {
    fetch_version_metadata(CUDNN_BASE_URL, "cuDNN", version).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_available_versions_from_html() {
        let html = r#"
<!DOCTYPE html>
<html>
<head><title>Index of /compute/cuda/redist/</title></head>
<body>
<h1>Index of /compute/cuda/redist/</h1>
<pre>
<a href="../">../</a>
<a href="redistrib_11.8.0.json">redistrib_11.8.0.json</a>              2023-10-15 12:00  123456
<a href="redistrib_12.0.0.json">redistrib_12.0.0.json</a>              2023-12-01 12:00  234567
<a href="redistrib_12.4.1.json">redistrib_12.4.1.json</a>              2024-06-01 12:00  789012
</pre>
</body>
</html>
        "#;

        let versions = parse_available_versions(html);
        assert_eq!(versions.len(), 3);
        assert!(versions.contains("11.8.0"));
        assert!(versions.contains("12.0.0"));
        assert!(versions.contains("12.4.1"));
    }

    #[test]
    fn test_parse_available_versions_empty() {
        let html = "<html><body>No versions here</body></html>";
        let versions = parse_available_versions(html);
        assert!(versions.is_empty());
    }

    #[test]
    fn test_parse_available_versions_sorted() {
        let html = r#"
<a href="redistrib_12.4.1.json">redistrib_12.4.1.json</a>
<a href="redistrib_11.8.0.json">redistrib_11.8.0.json</a>
<a href="redistrib_12.0.0.json">redistrib_12.0.0.json</a>
        "#;

        let versions = parse_available_versions(html);
        let versions_vec: Vec<&String> = versions.iter().collect();
        // BTreeSet keeps them sorted
        assert_eq!(versions_vec[0], "11.8.0");
        assert_eq!(versions_vec[1], "12.0.0");
        assert_eq!(versions_vec[2], "12.4.1");
    }

    #[test]
    fn test_parse_available_versions_ignores_invalid() {
        let html = r#"
<a href="redistrib_12.4.1.json">redistrib_12.4.1.json</a>
<a href="redistrib_invalid.json">redistrib_invalid.json</a>
<a href="some_other_file.txt">some_other_file.txt</a>
<a href="redistrib_12.json">redistrib_12.json</a>
<a href="redistrib_12.0.0.json">redistrib_12.0.0.json</a>
        "#;

        let versions = parse_available_versions(html);
        assert_eq!(versions.len(), 2);
        assert!(versions.contains("12.4.1"));
        assert!(versions.contains("12.0.0"));
    }

    #[test]
    fn test_base_download_urls() {
        assert!(CUDA_BASE_URL.contains("nvidia.com"));
        assert!(CUDA_BASE_URL.contains("cuda"));
        assert!(CUDNN_BASE_URL.contains("nvidia.com"));
        assert!(CUDNN_BASE_URL.contains("cudnn"));
    }
}
