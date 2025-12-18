use std::cmp::Reverse;

use anyhow::Result;

use crate::cuda::discover::{CUDA_BASE_URL, CUDNN_BASE_URL, find_newest_compatible_cudnn};
use crate::cuda::metadata::{CudaReleaseMetadata, PlatformInfo};
use crate::cuda::version::CudaVersion;

use super::download::DownloadTask;

fn parse_size(size_str: &str, package_name: &str) -> Option<u64> {
    size_str
        .parse()
        .inspect_err(|_| log::warn!("Failed to parse size '{}' for {}", size_str, package_name))
        .ok()
}

pub async fn find_compatible_cudnn(cuda_version: &CudaVersion) -> Result<Option<(String, String)>> {
    if let Some(cudnn_version) = find_newest_compatible_cudnn(cuda_version.as_str()).await? {
        let cuda_variant = format!("cuda{}", cuda_version.major());
        return Ok(Some((cudnn_version, cuda_variant)));
    }

    Ok(None)
}

pub fn collect_cuda_download_tasks(
    metadata: &CudaReleaseMetadata,
    cuda_version: &CudaVersion,
    platform: &str,
) -> Vec<DownloadTask> {
    let mut tasks = Vec::with_capacity(metadata.packages.len());
    let variant_key = format!("cuda{}", cuda_version.major());

    for (package_name, package_info) in &metadata.packages {
        if package_name.starts_with("release_") {
            continue;
        }

        let Some(platform_info) = package_info.get_platform(platform) else {
            continue;
        };

        let download_info = match platform_info {
            PlatformInfo::Simple(info) => info,
            PlatformInfo::Variants(variants) => match variants.get(&variant_key) {
                Some(info) => info,
                None => continue,
            },
        };

        let url = format!("{}/{}", CUDA_BASE_URL, download_info.relative_path);
        let size = parse_size(&download_info.size, package_name);

        tasks.push(DownloadTask {
            package_name: package_name.clone(),
            url,
            sha256: download_info.sha256.clone(),
            size,
            relative_path: download_info.relative_path.clone(),
        });
    }

    // Sort by size descending, with unknown sizes (None) at the end
    tasks.sort_unstable_by_key(|t| Reverse(t.size));

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
        url,
        sha256: download_info.sha256.clone(),
        size,
        relative_path: download_info.relative_path.clone(),
    })
}
