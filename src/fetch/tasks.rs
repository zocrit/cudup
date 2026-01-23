use anyhow::{Context, Result};

use crate::cuda::discover::{CUDA_BASE_URL, CUDNN_BASE_URL, find_newest_compatible_cudnn};
use crate::cuda::metadata::{CudaReleaseMetadata, PlatformInfo};

use super::download::DownloadTask;
use super::utils::TARGET_PLATFORM;

/// Parses size string, logging a warning and returning 0 on failure
fn parse_size(size_str: &str, package_name: &str) -> u64 {
    size_str.parse().unwrap_or_else(|e| {
        log::warn!(
            "Failed to parse size '{}' for {}: {}",
            size_str,
            package_name,
            e
        );
        0
    })
}

/// Finds the best compatible cuDNN version for a given CUDA version
///
/// Returns (cudnn_version, cuda_variant) tuple
pub async fn find_compatible_cudnn(cuda_version: &str) -> Result<Option<(String, String)>> {
    let cuda_major = cuda_version
        .split('.')
        .next()
        .context("Invalid CUDA version format")?;

    if let Some(cudnn_version) = find_newest_compatible_cudnn(cuda_version).await? {
        let cuda_variant = format!("cuda{}", cuda_major);
        return Ok(Some((cudnn_version, cuda_variant)));
    }

    Ok(None)
}

pub fn collect_cuda_download_tasks(
    metadata: &CudaReleaseMetadata,
    cuda_version: &str,
) -> Vec<DownloadTask> {
    let mut tasks = Vec::with_capacity(metadata.packages.len());

    for (package_name, package_info) in &metadata.packages {
        if package_name.starts_with("release_") {
            continue;
        }

        let Some(platform_info) = package_info.get_platform(TARGET_PLATFORM) else {
            continue;
        };

        let download_info = match platform_info {
            PlatformInfo::Simple(info) => info,
            PlatformInfo::Variants(variants) => {
                let cuda_major = cuda_version.split('.').next().unwrap_or("12");
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

    tasks.sort_unstable_by(|a, b| b.size.cmp(&a.size));

    tasks
}

pub fn collect_cudnn_download_task(
    metadata: &CudaReleaseMetadata,
    cuda_variant: &str,
) -> Option<DownloadTask> {
    let cudnn_pkg = metadata.get_package("cudnn")?;
    let platform_info = cudnn_pkg.get_platform(TARGET_PLATFORM)?;

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
                    }
                }
            }"#,
        )
        .unwrap()
    }

    #[test]
    fn test_collect_cuda_download_tasks() {
        let metadata = sample_cuda_metadata();
        let tasks = collect_cuda_download_tasks(&metadata, "12.4.1");

        // Should have 2 packages (cuda_cccl and cuda_cudart), release_label is skipped
        assert_eq!(tasks.len(), 2);

        let cccl = tasks
            .iter()
            .find(|t| t.package_name == "cuda_cccl")
            .expect("cuda_cccl task should exist");
        assert_eq!(cccl.size, 1234567);
        assert!(cccl.url.contains("cuda_cccl-linux-x86_64"));
        assert!(!cccl.sha256.is_empty());
    }

    #[test]
    fn test_collect_cuda_download_tasks_skips_release_packages() {
        let metadata = sample_cuda_metadata();
        let tasks = collect_cuda_download_tasks(&metadata, "12.4.1");

        // release_label package should be skipped
        assert!(!tasks.iter().any(|t| t.package_name.starts_with("release_")));
    }

    #[test]
    fn test_collect_cuda_download_tasks_sorted_by_size_descending() {
        let metadata = sample_cuda_metadata();
        let tasks = collect_cuda_download_tasks(&metadata, "12.4.1");

        // Tasks should be sorted largest first
        // cuda_cudart (3456789) > cuda_cccl (1234567)
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].package_name, "cuda_cudart");
        assert_eq!(tasks[1].package_name, "cuda_cccl");

        // Verify sizes are in descending order
        for i in 1..tasks.len() {
            assert!(tasks[i - 1].size >= tasks[i].size);
        }
    }

    #[test]
    fn test_collect_cudnn_download_task_cuda12() {
        let metadata = sample_cudnn_metadata();
        let task =
            collect_cudnn_download_task(&metadata, "cuda12").expect("should find cuda12 task");

        assert_eq!(task.package_name, "cudnn");
        assert_eq!(task.size, 987654322);
        assert!(task.url.contains("cuda12-archive"));
        assert!(task.relative_path.contains("cuda12"));
    }

    #[test]
    fn test_collect_cudnn_download_task_cuda11() {
        let metadata = sample_cudnn_metadata();
        let task =
            collect_cudnn_download_task(&metadata, "cuda11").expect("should find cuda11 task");

        assert!(task.url.contains("cuda11-archive"));
    }

    #[test]
    fn test_collect_cudnn_download_task_invalid_variant() {
        let metadata = sample_cudnn_metadata();
        let task = collect_cudnn_download_task(&metadata, "cuda10");

        // CUDA 10 is not supported
        assert!(task.is_none());
    }

    #[test]
    fn test_collect_cudnn_download_task_no_cudnn_package() {
        let metadata = sample_cuda_metadata(); // CUDA metadata, no cuDNN
        let task = collect_cudnn_download_task(&metadata, "cuda12");

        assert!(task.is_none());
    }
}
