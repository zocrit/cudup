use crate::cuda::metadata::CudaReleaseMetadata;
use anyhow::{Context, Result};
use reqwest::Client;
use std::collections::BTreeSet;
use std::sync::LazyLock;

static VERSION_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"redistrib_(\d+\.\d+\.\d+)\.json").unwrap());

pub const CUDA_BASE_URL: &str = "https://developer.download.nvidia.com/compute/cuda/redist";
pub const CUDNN_BASE_URL: &str = "https://developer.download.nvidia.com/compute/cudnn/redist";

/// Fetches the list of available CUDA versions from NVIDIA's redist index
pub async fn fetch_available_cuda_versions() -> Result<BTreeSet<String>> {
    let client = Client::new();
    let response = client
        .get(format!("{}/", CUDA_BASE_URL))
        .send()
        .await
        .context("Failed to fetch CUDA versions index")?;

    let body = response
        .text()
        .await
        .context("Failed to read CUDA versions response")?;

    parse_available_versions(&body)
}

/// Fetches the list of available cuDNN versions from NVIDIA's redist index
pub async fn fetch_available_cudnn_versions() -> Result<BTreeSet<String>> {
    let client = Client::new();
    let response = client
        .get(format!("{}/", CUDNN_BASE_URL))
        .send()
        .await
        .context("Failed to fetch cuDNN versions index")?;

    let body = response
        .text()
        .await
        .context("Failed to read cuDNN versions response")?;

    parse_available_versions(&body)
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
pub async fn fetch_cuda_version_metadata(version: &str) -> Result<CudaReleaseMetadata> {
    let client = Client::new();
    let url = format!("{}/redistrib_{}.json", CUDA_BASE_URL, version);

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

    response
        .json()
        .await
        .with_context(|| format!("Failed to parse CUDA {} metadata JSON", version))
}

/// Fetches cuDNN versions compatible with a specific CUDA version
///
/// Returns a map of cuDNN version to its corresponding CUDA variant string (e.g., "cuda12")
/// Only returns cuDNN versions that support the given CUDA major version
pub async fn fetch_compatible_cudnn_versions(cuda_version: &str) -> Result<BTreeSet<String>> {
    let cuda_major = cuda_version
        .split('.')
        .next()
        .context("Invalid CUDA version format")?;

    let all_cudnn_versions = fetch_available_cudnn_versions().await?;
    let mut compatible_versions = BTreeSet::new();

    for cudnn_version in &all_cudnn_versions {
        let metadata = match fetch_cudnn_version_metadata(cudnn_version).await {
            Ok(m) => m,
            Err(_) => continue, // Skip versions we can't fetch
        };

        // Check if this cuDNN supports our CUDA major version
        if let Some(cudnn_pkg) = metadata.get_package("cudnn")
            && let Some(variants) = &cudnn_pkg.cuda_variant
            && variants.contains(&cuda_major.to_string())
        {
            compatible_versions.insert(cudnn_version.clone());
        }
    }

    Ok(compatible_versions)
}

/// Fetches the detailed metadata JSON for a specific cuDNN version
pub async fn fetch_cudnn_version_metadata(version: &str) -> Result<CudaReleaseMetadata> {
    let client = Client::new();
    let url = format!("{}/redistrib_{}.json", CUDNN_BASE_URL, version);

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

    response
        .json()
        .await
        .with_context(|| format!("Failed to parse cuDNN {} metadata JSON", version))
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

        let versions = parse_available_versions(html).unwrap();
        assert_eq!(versions.len(), 3);
        assert!(versions.contains("11.8.0"));
        assert!(versions.contains("12.0.0"));
        assert!(versions.contains("12.4.1"));
    }

    #[test]
    fn test_parse_available_versions_empty() {
        let html = "<html><body>No versions here</body></html>";
        let versions = parse_available_versions(html).unwrap();
        assert!(versions.is_empty());
    }

    #[test]
    fn test_parse_available_versions_sorted() {
        let html = r#"
<a href="redistrib_12.4.1.json">redistrib_12.4.1.json</a>
<a href="redistrib_11.8.0.json">redistrib_11.8.0.json</a>
<a href="redistrib_12.0.0.json">redistrib_12.0.0.json</a>
        "#;

        let versions = parse_available_versions(html).unwrap();
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

        let versions = parse_available_versions(html).unwrap();
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
