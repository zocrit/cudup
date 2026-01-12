use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tokio::fs;

use crate::cuda::metadata::CudaReleaseMetadata;

/// Cache TTL for version lists (24 hours)
const VERSION_LIST_TTL: Duration = Duration::from_secs(24 * 60 * 60);

/// Cache TTL for metadata JSONs (7 days)
const METADATA_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// Cached version list with timestamp
#[derive(Debug, Serialize, Deserialize)]
pub struct CachedVersionList {
    pub versions: BTreeSet<String>,
    pub cached_at: u64, // Unix timestamp
}

/// Cached metadata with timestamp
#[derive(Debug, Serialize, Deserialize)]
pub struct CachedMetadata {
    pub metadata: CudaReleaseMetadata,
    pub cached_at: u64, // Unix timestamp
}

/// Returns the base cudup directory (~/.cudup)
pub fn cudup_home() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".cudup"))
}

/// Returns the cache directory (~/.cudup/cache)
pub fn cache_dir() -> Result<PathBuf> {
    Ok(cudup_home()?.join("cache"))
}

/// Returns the versions directory (~/.cudup/versions)
pub fn versions_dir() -> Result<PathBuf> {
    Ok(cudup_home()?.join("versions"))
}

/// Ensures the cache directory structure exists
pub async fn ensure_cache_dirs() -> Result<()> {
    let cache = cache_dir()?;
    fs::create_dir_all(&cache).await?;
    fs::create_dir_all(cache.join("cuda")).await?;
    fs::create_dir_all(cache.join("cudnn")).await?;
    Ok(())
}

/// Returns the current Unix timestamp
fn now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Checks if a cached timestamp is still valid given a TTL
fn is_cache_valid(cached_at: u64, ttl: Duration) -> bool {
    let now = now_timestamp();
    now.saturating_sub(cached_at) < ttl.as_secs()
}

// ============================================================================
// Version List Caching
// ============================================================================

/// Gets the path for a cached version list
fn version_list_path(product: &str) -> Result<PathBuf> {
    Ok(cache_dir()?.join(format!("{}_versions.json", product)))
}

/// Loads cached CUDA versions if valid
pub async fn load_cached_cuda_versions(force_refresh: bool) -> Result<Option<BTreeSet<String>>> {
    load_cached_versions("cuda", force_refresh).await
}

/// Loads cached cuDNN versions if valid
pub async fn load_cached_cudnn_versions(force_refresh: bool) -> Result<Option<BTreeSet<String>>> {
    load_cached_versions("cudnn", force_refresh).await
}

/// Generic loader for cached version lists
async fn load_cached_versions(
    product: &str,
    force_refresh: bool,
) -> Result<Option<BTreeSet<String>>> {
    if force_refresh {
        return Ok(None);
    }

    let path = version_list_path(product)?;

    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path).await?;
    let cached: CachedVersionList = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse cached {} versions", product))?;

    if is_cache_valid(cached.cached_at, VERSION_LIST_TTL) {
        Ok(Some(cached.versions))
    } else {
        Ok(None)
    }
}

/// Saves CUDA versions to cache
pub async fn save_cuda_versions(versions: &BTreeSet<String>) -> Result<()> {
    save_versions("cuda", versions).await
}

/// Saves cuDNN versions to cache
pub async fn save_cudnn_versions(versions: &BTreeSet<String>) -> Result<()> {
    save_versions("cudnn", versions).await
}

/// Generic saver for version lists
async fn save_versions(product: &str, versions: &BTreeSet<String>) -> Result<()> {
    ensure_cache_dirs().await?;

    let cached = CachedVersionList {
        versions: versions.clone(),
        cached_at: now_timestamp(),
    };

    let path = version_list_path(product)?;
    let content = serde_json::to_string_pretty(&cached)?;
    fs::write(&path, content).await?;

    Ok(())
}

// ============================================================================
// Metadata Caching
// ============================================================================

/// Gets the path for cached metadata
fn metadata_path(product: &str, version: &str) -> Result<PathBuf> {
    Ok(cache_dir()?.join(product).join(format!("{}.json", version)))
}

/// Loads cached CUDA metadata if valid
pub async fn load_cached_cuda_metadata(
    version: &str,
    force_refresh: bool,
) -> Result<Option<CudaReleaseMetadata>> {
    load_cached_metadata("cuda", version, force_refresh).await
}

/// Loads cached cuDNN metadata if valid
pub async fn load_cached_cudnn_metadata(
    version: &str,
    force_refresh: bool,
) -> Result<Option<CudaReleaseMetadata>> {
    load_cached_metadata("cudnn", version, force_refresh).await
}

/// Generic loader for cached metadata
async fn load_cached_metadata(
    product: &str,
    version: &str,
    force_refresh: bool,
) -> Result<Option<CudaReleaseMetadata>> {
    if force_refresh {
        return Ok(None);
    }

    let path = metadata_path(product, version)?;

    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path).await?;
    let cached: CachedMetadata = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse cached {} {} metadata", product, version))?;

    if is_cache_valid(cached.cached_at, METADATA_TTL) {
        Ok(Some(cached.metadata))
    } else {
        Ok(None)
    }
}

/// Saves CUDA metadata to cache
pub async fn save_cuda_metadata(version: &str, metadata: &CudaReleaseMetadata) -> Result<()> {
    save_metadata("cuda", version, metadata).await
}

/// Saves cuDNN metadata to cache
pub async fn save_cudnn_metadata(version: &str, metadata: &CudaReleaseMetadata) -> Result<()> {
    save_metadata("cudnn", version, metadata).await
}

/// Generic saver for metadata
async fn save_metadata(product: &str, version: &str, metadata: &CudaReleaseMetadata) -> Result<()> {
    ensure_cache_dirs().await?;

    let cached = CachedMetadata {
        metadata: metadata.clone(),
        cached_at: now_timestamp(),
    };

    let path = metadata_path(product, version)?;
    let content = serde_json::to_string_pretty(&cached)?;
    fs::write(&path, content).await?;

    Ok(())
}

// ============================================================================
// Cache Management
// ============================================================================

/// Clears all cached data
pub async fn clear_cache() -> Result<()> {
    let cache = cache_dir()?;
    if cache.exists() {
        fs::remove_dir_all(&cache).await?;
    }
    ensure_cache_dirs().await?;
    Ok(())
}

/// Returns cache statistics
pub async fn cache_stats() -> Result<CacheStats> {
    let cache = cache_dir()?;

    let cuda_versions = version_list_path("cuda")?.exists();
    let cudnn_versions = version_list_path("cudnn")?.exists();

    let cuda_metadata_count = count_files_in_dir(&cache.join("cuda")).await;
    let cudnn_metadata_count = count_files_in_dir(&cache.join("cudnn")).await;

    Ok(CacheStats {
        cuda_versions_cached: cuda_versions,
        cudnn_versions_cached: cudnn_versions,
        cuda_metadata_count,
        cudnn_metadata_count,
    })
}

async fn count_files_in_dir(dir: &PathBuf) -> usize {
    if !dir.exists() {
        return 0;
    }

    let mut count = 0;
    if let Ok(mut entries) = fs::read_dir(dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if entry.path().extension().is_some_and(|ext| ext == "json") {
                count += 1;
            }
        }
    }
    count
}

#[derive(Debug)]
pub struct CacheStats {
    pub cuda_versions_cached: bool,
    pub cudnn_versions_cached: bool,
    pub cuda_metadata_count: usize,
    pub cudnn_metadata_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cudup_home() {
        let home = cudup_home().unwrap();
        assert!(home.to_string_lossy().contains(".cudup"));
    }

    #[test]
    fn test_cache_dir() {
        let cache = cache_dir().unwrap();
        assert!(cache.to_string_lossy().contains(".cudup"));
        assert!(cache.to_string_lossy().contains("cache"));
    }

    #[test]
    fn test_versions_dir() {
        let versions = versions_dir().unwrap();
        assert!(versions.to_string_lossy().contains(".cudup"));
        assert!(versions.to_string_lossy().contains("versions"));
    }

    #[test]
    fn test_now_timestamp() {
        let ts = now_timestamp();
        // Timestamp should be reasonable (after 2020)
        assert!(ts > 1577836800); // 2020-01-01
    }

    #[test]
    fn test_is_cache_valid_fresh() {
        let now = now_timestamp();
        // Cache from 1 hour ago with 24 hour TTL should be valid
        let cached_at = now - 3600;
        assert!(is_cache_valid(cached_at, Duration::from_secs(24 * 60 * 60)));
    }

    #[test]
    fn test_is_cache_valid_expired() {
        let now = now_timestamp();
        // Cache from 25 hours ago with 24 hour TTL should be invalid
        let cached_at = now - (25 * 60 * 60);
        assert!(!is_cache_valid(
            cached_at,
            Duration::from_secs(24 * 60 * 60)
        ));
    }

    #[test]
    fn test_is_cache_valid_edge() {
        let now = now_timestamp();
        // Cache exactly at TTL boundary
        let ttl = Duration::from_secs(100);
        let cached_at = now - 99;
        assert!(is_cache_valid(cached_at, ttl));

        let cached_at = now - 100;
        assert!(!is_cache_valid(cached_at, ttl));
    }

    #[test]
    fn test_cached_version_list_serialization() {
        let versions: BTreeSet<String> = ["12.0.0", "12.1.0", "12.2.0"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        let cached = CachedVersionList {
            versions: versions.clone(),
            cached_at: 1234567890,
        };

        let json = serde_json::to_string(&cached).unwrap();
        let deserialized: CachedVersionList = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.versions, versions);
        assert_eq!(deserialized.cached_at, 1234567890);
    }

    #[tokio::test]
    async fn test_save_and_load_cuda_versions() {
        // Ensure cache dirs exist
        ensure_cache_dirs().await.unwrap();

        let versions: BTreeSet<String> =
            ["12.0.0", "12.1.0"].iter().map(|s| s.to_string()).collect();

        // Save versions
        save_cuda_versions(&versions).await.unwrap();

        // Load without force refresh
        let loaded = load_cached_cuda_versions(false).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap(), versions);
    }

    #[tokio::test]
    async fn test_force_refresh_bypasses_cache() {
        ensure_cache_dirs().await.unwrap();

        let versions: BTreeSet<String> = ["11.0.0"].iter().map(|s| s.to_string()).collect();
        save_cuda_versions(&versions).await.unwrap();

        // Force refresh should return None
        let loaded = load_cached_cuda_versions(true).await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_ensure_cache_dirs_creates_structure() {
        ensure_cache_dirs().await.unwrap();

        let cache = cache_dir().unwrap();
        assert!(cache.exists());
        assert!(cache.join("cuda").exists());
        assert!(cache.join("cudnn").exists());
    }
}
