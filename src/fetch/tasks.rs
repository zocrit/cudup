use anyhow::{Context, Result};

use crate::cuda::discover::{CUDA_BASE_URL, CUDNN_BASE_URL, find_newest_compatible_cudnn};
use crate::cuda::metadata::{CudaReleaseMetadata, PlatformInfo};

use super::download::DownloadTask;

/// Parses size string, returning None if parsing fails
fn parse_size(size_str: &str, package_name: &str) -> Option<u64> {
    size_str.parse().ok().or_else(|| {
        log::warn!("Failed to parse size '{}' for {}", size_str, package_name);
        None
    })
}

/// Extracts the major version from a CUDA version string (e.g., "12.4.1" -> "12")
fn cuda_major_version(cuda_version: &str) -> Option<&str> {
    cuda_version.split('.').next().filter(|s| !s.is_empty())
}

/// Finds the best compatible cuDNN version for a given CUDA version
///
/// Returns (cudnn_version, cuda_variant) tuple
pub async fn find_compatible_cudnn(cuda_version: &str) -> Result<Option<(String, String)>> {
    let cuda_major =
        cuda_major_version(cuda_version).context("Invalid CUDA version format")?;

    if let Some(cudnn_version) = find_newest_compatible_cudnn(cuda_version).await? {
        let cuda_variant = format!("cuda{}", cuda_major);
        return Ok(Some((cudnn_version, cuda_variant)));
    }

    Ok(None)
}

pub fn collect_cuda_download_tasks(
    metadata: &CudaReleaseMetadata,
    cuda_version: &str,
    platform: &str,
) -> Vec<DownloadTask> {
    let mut tasks = Vec::with_capacity(metadata.packages.len());

    for (package_name, package_info) in &metadata.packages {
        if package_name.starts_with("release_") {
            continue;
        }

        let Some(platform_info) = package_info.get_platform(platform) else {
            continue;
        };

        let download_info = match platform_info {
            PlatformInfo::Simple(info) => info,
            PlatformInfo::Variants(variants) => {
                let cuda_major = cuda_major_version(cuda_version)
                    .expect("CUDA version should have been validated at CLI entry");
                let variant_key = format!("cuda{}", cuda_major);
                match variants.get(&variant_key) {
                    Some(info) => info,
                    None => continue,
                }
            }
        };

        let url = format!("{}/{}", CUDA_BASE_URL, download_info.relative_path);
        let size = parse_size(&download_info.size, package_name);

        tasks.push(DownloadTask {
            package_name: package_name.clone(),
            version: package_info.version.clone(),
            url,
            sha256: download_info.sha256.clone(),
            size,
            relative_path: download_info.relative_path.clone(),
        });
    }

    // Sort by size descending, with unknown sizes (None) at the end
    tasks.sort_unstable_by(|a, b| match (b.size, a.size) {
        (Some(b_size), Some(a_size)) => b_size.cmp(&a_size),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });

    tasks
}

pub fn collect_cudnn_download_task(
    metadata: &CudaReleaseMetadata,
    cuda_variant: &str,
    platform: &str,
) -> Option<DownloadTask> {
    let cudnn_pkg = metadata.get_package("cudnn")?;
    let platform_info = cudnn_pkg.get_platform(platform)?;

    let download_info = match platform_info {
        PlatformInfo::Simple(info) => info,
        PlatformInfo::Variants(variants) => variants.get(cuda_variant)?,
    };

    let url = format!("{}/{}", CUDNN_BASE_URL, download_info.relative_path);
    let size = parse_size(&download_info.size, "cudnn");

    Some(DownloadTask {
        package_name: "cudnn".to_string(),
        version: cudnn_pkg.version.clone(),
        url,
        sha256: download_info.sha256.clone(),
        size,
        relative_path: download_info.relative_path.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cuda::metadata::CudaReleaseMetadata;

    fn sample_cuda_metadata() -> CudaReleaseMetadata {
        serde_json::from_str(
            r#"{
                "release_date": "2024-06-01",
                "cuda_cccl": {
                    "name": "CUDA C++ Core Libraries",
                    "license": "NVIDIA Software License",
                    "version": "12.4.127",
                    "linux-x86_64": {
                        "relative_path": "cuda_cccl/linux-x86_64/cuda_cccl-linux-x86_64-12.4.127-archive.tar.xz",
                        "sha256": "abc123def456789012345678901234567890123456789012345678901234abcd",
                        "md5": "abc123def456",
                        "size": "1234567"
                    },
                    "linux-sbsa": {
                        "relative_path": "cuda_cccl/linux-sbsa/cuda_cccl-linux-sbsa-12.4.127-archive.tar.xz",
                        "sha256": "sbsa_abc123def456789012345678901234567890123456789012345678901234",
                        "md5": "sbsa_abc123",
                        "size": "1234568"
                    }
                },
                "cuda_cudart": {
                    "name": "CUDA Runtime",
                    "license": "NVIDIA Software License",
                    "version": "12.4.127",
                    "linux-x86_64": {
                        "relative_path": "cuda_cudart/linux-x86_64/cuda_cudart-linux-x86_64-12.4.127-archive.tar.xz",
                        "sha256": "789012345678901234567890123456789012345678901234567890123456789a",
                        "md5": "789012345678",
                        "size": "3456789"
                    },
                    "linux-sbsa": {
                        "relative_path": "cuda_cudart/linux-sbsa/cuda_cudart-linux-sbsa-12.4.127-archive.tar.xz",
                        "sha256": "sbsa_789012345678901234567890123456789012345678901234567890123456",
                        "md5": "sbsa_789012",
                        "size": "3456790"
                    }
                },
                "release_notes": {
                    "name": "Release Notes",
                    "license": "NVIDIA Software License",
                    "version": "12.4.1",
                    "linux-x86_64": {
                        "relative_path": "release_notes/linux-x86_64/release_notes-linux-x86_64-12.4.1-archive.tar.xz",
                        "sha256": "releasenotes123456789012345678901234567890123456789012345678901234",
                        "md5": "releasenotes12",
                        "size": "12345"
                    }
                }
            }"#,
        )
        .unwrap()
    }

    fn sample_cudnn_metadata() -> CudaReleaseMetadata {
        serde_json::from_str(
            r#"{
                "release_date": "2024-05-15",
                "release_label": "9.1.0",
                "release_product": "cudnn",
                "cudnn": {
                    "name": "cuDNN",
                    "license": "NVIDIA cuDNN Software License",
                    "license_path": "cudnn/LICENSE.txt",
                    "version": "9.1.0.70",
                    "cuda_variant": ["11", "12"],
                    "linux-x86_64": {
                        "cuda11": {
                            "relative_path": "cudnn/linux-x86_64/cudnn-linux-x86_64-9.1.0.70_cuda11-archive.tar.xz",
                            "sha256": "cudnn11sha256hash012345678901234567890123456789012345678901234567",
                            "md5": "cudnn11md5hash",
                            "size": "987654321"
                        },
                        "cuda12": {
                            "relative_path": "cudnn/linux-x86_64/cudnn-linux-x86_64-9.1.0.70_cuda12-archive.tar.xz",
                            "sha256": "cudnn12sha256hash012345678901234567890123456789012345678901234567",
                            "md5": "cudnn12md5hash",
                            "size": "987654322"
                        }
                    },
                    "linux-sbsa": {
                        "cuda11": {
                            "relative_path": "cudnn/linux-sbsa/cudnn-linux-sbsa-9.1.0.70_cuda11-archive.tar.xz",
                            "sha256": "sbsa_cudnn11sha256hash0123456789012345678901234567890123456789012",
                            "md5": "sbsa_cudnn11md5",
                            "size": "987654323"
                        },
                        "cuda12": {
                            "relative_path": "cudnn/linux-sbsa/cudnn-linux-sbsa-9.1.0.70_cuda12-archive.tar.xz",
                            "sha256": "sbsa_cudnn12sha256hash0123456789012345678901234567890123456789012",
                            "md5": "sbsa_cudnn12md5",
                            "size": "987654324"
                        }
                    }
                }
            }"#,
        )
        .unwrap()
    }

    const TEST_PLATFORM: &str = "linux-x86_64";

    #[test]
    fn test_collect_cuda_download_tasks() {
        let metadata = sample_cuda_metadata();
        let tasks = collect_cuda_download_tasks(&metadata, "12.4.1", TEST_PLATFORM);

        // Should have 2 packages (cuda_cccl and cuda_cudart), release_label is skipped
        assert_eq!(tasks.len(), 2);

        let cccl = tasks
            .iter()
            .find(|t| t.package_name == "cuda_cccl")
            .expect("cuda_cccl task should exist");
        assert_eq!(cccl.size, Some(1234567));
        assert!(cccl.url.contains("cuda_cccl-linux-x86_64"));
        assert!(!cccl.sha256.is_empty());
    }

    #[test]
    fn test_collect_cuda_download_tasks_skips_release_packages() {
        let metadata = sample_cuda_metadata();
        let tasks = collect_cuda_download_tasks(&metadata, "12.4.1", TEST_PLATFORM);

        // release_label package should be skipped
        assert!(!tasks.iter().any(|t| t.package_name.starts_with("release_")));
    }

    #[test]
    fn test_collect_cuda_download_tasks_sorted_by_size_descending() {
        let metadata = sample_cuda_metadata();
        let tasks = collect_cuda_download_tasks(&metadata, "12.4.1", TEST_PLATFORM);

        // Tasks should be sorted largest first
        // cuda_cudart (3456789) > cuda_cccl (1234567)
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].package_name, "cuda_cudart");
        assert_eq!(tasks[1].package_name, "cuda_cccl");

        // Verify sizes are in descending order (known sizes)
        for i in 1..tasks.len() {
            match (tasks[i - 1].size, tasks[i].size) {
                (Some(a), Some(b)) => assert!(a >= b),
                (Some(_), None) => {} // Known before unknown is correct
                (None, Some(_)) => panic!("Unknown size should not come before known size"),
                (None, None) => {}
            }
        }
    }

    #[test]
    fn test_collect_cudnn_download_task_cuda12() {
        let metadata = sample_cudnn_metadata();
        let task = collect_cudnn_download_task(&metadata, "cuda12", TEST_PLATFORM)
            .expect("should find cuda12 task");

        assert_eq!(task.package_name, "cudnn");
        assert_eq!(task.size, Some(987654322));
        assert!(task.url.contains("cuda12-archive"));
        assert!(task.relative_path.contains("cuda12"));
    }

    #[test]
    fn test_collect_cudnn_download_task_cuda11() {
        let metadata = sample_cudnn_metadata();
        let task = collect_cudnn_download_task(&metadata, "cuda11", TEST_PLATFORM)
            .expect("should find cuda11 task");

        assert!(task.url.contains("cuda11-archive"));
    }

    #[test]
    fn test_collect_cudnn_download_task_invalid_variant() {
        let metadata = sample_cudnn_metadata();
        let task = collect_cudnn_download_task(&metadata, "cuda10", TEST_PLATFORM);

        // CUDA 10 is not supported
        assert!(task.is_none());
    }

    #[test]
    fn test_collect_cudnn_download_task_no_cudnn_package() {
        let metadata = sample_cuda_metadata(); // CUDA metadata, no cuDNN
        let task = collect_cudnn_download_task(&metadata, "cuda12", TEST_PLATFORM);

        assert!(task.is_none());
    }

    #[test]
    fn test_collect_cuda_download_tasks_unsupported_platform() {
        let metadata = sample_cuda_metadata();
        let tasks = collect_cuda_download_tasks(&metadata, "12.4.1", "linux-ppc64le");

        // No packages for this platform in test data
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_collect_cuda_download_tasks_arm64() {
        let metadata = sample_cuda_metadata();
        let tasks = collect_cuda_download_tasks(&metadata, "12.4.1", "linux-sbsa");

        // Should have 2 packages for ARM64
        assert_eq!(tasks.len(), 2);

        let cccl = tasks
            .iter()
            .find(|t| t.package_name == "cuda_cccl")
            .expect("cuda_cccl task should exist");
        assert!(cccl.url.contains("linux-sbsa"));
        assert!(cccl.relative_path.contains("linux-sbsa"));
        assert_eq!(cccl.size, Some(1234568));
    }

    #[test]
    fn test_collect_cudnn_download_task_arm64() {
        let metadata = sample_cudnn_metadata();
        let task = collect_cudnn_download_task(&metadata, "cuda12", "linux-sbsa")
            .expect("should find cuda12 task for ARM64");

        assert_eq!(task.package_name, "cudnn");
        assert_eq!(task.size, Some(987654324));
        assert!(task.url.contains("linux-sbsa"));
        assert!(task.url.contains("cuda12-archive"));
    }

    #[test]
    fn test_cuda_major_version() {
        assert_eq!(cuda_major_version("12.4.1"), Some("12"));
        assert_eq!(cuda_major_version("11.8.0"), Some("11"));
        assert_eq!(cuda_major_version("10"), Some("10"));
        assert_eq!(cuda_major_version(""), None);
    }

    #[test]
    fn test_cuda_major_version_edge_cases() {
        // Single component
        assert_eq!(cuda_major_version("12"), Some("12"));
        // Two components
        assert_eq!(cuda_major_version("12.4"), Some("12"));
        // Trailing dot
        assert_eq!(cuda_major_version("12."), Some("12"));
    }
}
