use anyhow::{Context, Result, bail};
use reqwest::Client;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::config;
use crate::cuda::discover::{
    BaseDownloadUrls, fetch_available_cuda_versions, fetch_compatible_cudnn_versions,
    fetch_cuda_version_metadata, fetch_cudnn_version_metadata,
};
use crate::cuda::metadata::{CudaReleaseMetadata, PlatformInfo};

const TARGET_PLATFORM: &str = "linux-x86_64";

#[derive(Debug, Clone)]
pub struct DownloadTask {
    pub package_name: String,
    pub version: String,
    pub url: String,
    pub sha256: String,
    pub size: u64,
    pub relative_path: String,
}

/// Finds the best compatible cuDNN version for a given CUDA version
///
/// Returns (cudnn_version, cuda_variant) tuple
pub async fn find_compatible_cudnn(cuda_version: &str) -> Result<Option<(String, String)>> {
    let cuda_major = cuda_version
        .split('.')
        .next()
        .context("Invalid CUDA version format")?;

    let compatible_versions = fetch_compatible_cudnn_versions(cuda_version).await?;

    // Return the newest compatible version (last in the sorted set)
    if let Some(cudnn_version) = compatible_versions.iter().next_back() {
        let cuda_variant = format!("cuda{}", cuda_major);
        return Ok(Some((cudnn_version.clone(), cuda_variant)));
    }

    Ok(None)
}

pub fn collect_cuda_download_tasks(
    metadata: &CudaReleaseMetadata,
    cuda_version: &str,
) -> Result<Vec<DownloadTask>> {
    let mut tasks = Vec::new();

    for (package_name, package_info) in &metadata.packages {
        if package_name.starts_with("release_") {
            continue;
        }

        // Get platform-specific download info
        let platform_info = match package_info.get_platform(TARGET_PLATFORM) {
            Some(info) => info,
            None => continue, // Package not available for this platform
        };

        let download_info = match platform_info {
            PlatformInfo::Simple(info) => info,
            PlatformInfo::Variants(variants) => {
                // For packages with variants, try to find one matching our CUDA version
                let cuda_major = cuda_version.split('.').next().unwrap_or("12");
                let variant_key = format!("cuda{}", cuda_major);
                match variants.get(&variant_key) {
                    Some(info) => info,
                    None => continue, // No compatible variant
                }
            }
        };

        let url = format!(
            "{}/{}",
            BaseDownloadUrls::cuda(),
            download_info.relative_path
        );

        let size = download_info.size.parse().unwrap_or(0);

        tasks.push(DownloadTask {
            package_name: package_name.clone(),
            version: package_info.version.clone(),
            url,
            sha256: download_info.sha256.clone(),
            size,
            relative_path: download_info.relative_path.clone(),
        });
    }

    Ok(tasks)
}

pub fn collect_cudnn_download_task(
    metadata: &CudaReleaseMetadata,
    cuda_variant: &str,
) -> Result<Option<DownloadTask>> {
    let cudnn_pkg = match metadata.get_package("cudnn") {
        Some(pkg) => pkg,
        None => return Ok(None),
    };

    let platform_info = match cudnn_pkg.get_platform(TARGET_PLATFORM) {
        Some(info) => info,
        None => return Ok(None),
    };

    let download_info = match platform_info {
        PlatformInfo::Simple(info) => info,
        PlatformInfo::Variants(variants) => match variants.get(cuda_variant) {
            Some(info) => info,
            None => return Ok(None),
        },
    };

    let url = format!(
        "{}/{}",
        BaseDownloadUrls::cudnn(),
        download_info.relative_path
    );

    let size = download_info.size.parse().unwrap_or(0);

    Ok(Some(DownloadTask {
        package_name: "cudnn".to_string(),
        version: cudnn_pkg.version.clone(),
        url,
        sha256: download_info.sha256.clone(),
        size,
        relative_path: download_info.relative_path.clone(),
    }))
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

pub async fn verify_checksum(path: &Path, expected_sha256: &str) -> Result<bool> {
    use sha2::{Digest, Sha256};
    use tokio::io::AsyncReadExt;

    // Normalize expected hash: trim whitespace and convert to lowercase
    let expected = expected_sha256.trim().to_lowercase();

    // Stream the file to avoid loading it entirely into memory
    let mut file = fs::File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 8192]; // 8KB buffer

    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    let actual = format!("{:x}", result);

    Ok(actual == expected)
}

pub async fn extract_tarball(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    use std::process::Stdio;
    use tokio::process::Command;

    fs::create_dir_all(dest_dir).await?;

    // Determine compression type from extension
    let tar_args = if archive_path.extension().is_some_and(|ext| ext == "xz") {
        vec![
            "xf",
            archive_path.to_str().unwrap(),
            "-C",
            dest_dir.to_str().unwrap(),
            "--strip-components=1",
        ]
    } else if archive_path.extension().is_some_and(|ext| ext == "gz") {
        vec![
            "xzf",
            archive_path.to_str().unwrap(),
            "-C",
            dest_dir.to_str().unwrap(),
            "--strip-components=1",
        ]
    } else {
        vec![
            "xf",
            archive_path.to_str().unwrap(),
            "-C",
            dest_dir.to_str().unwrap(),
            "--strip-components=1",
        ]
    };

    let output = Command::new("tar")
        .args(&tar_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to run tar command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to extract {}: {}", archive_path.display(), stderr);
    }

    Ok(())
}

/// Returns the installation directory for a specific CUDA version
pub fn version_install_dir(cuda_version: &str) -> Result<PathBuf> {
    Ok(config::versions_dir()?.join(cuda_version))
}

/// Returns the downloads directory for temporary archives
pub fn downloads_dir() -> Result<PathBuf> {
    config::downloads_dir()
}

/// Format bytes as human-readable size
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub async fn install_cuda_version(version: &str) -> Result<()> {
    let available_versions = fetch_available_cuda_versions().await?;
    if !available_versions.contains(version) {
        bail!(
            "CUDA version {} is not available. Use 'cudup list' to see available versions.",
            version
        );
    }

    let install_dir = version_install_dir(version)?;
    if install_dir.exists() {
        bail!(
            "CUDA {} is already installed at {}",
            version,
            install_dir.display()
        );
    }

    println!("Installing CUDA {} to {}", version, install_dir.display());

    // Fetch CUDA metadata
    println!("Fetching CUDA {} metadata...", version);
    let cuda_metadata = fetch_cuda_version_metadata(version).await?;

    // Collect CUDA download tasks
    let cuda_tasks = collect_cuda_download_tasks(&cuda_metadata, version)?;
    let cuda_total_size: u64 = cuda_tasks.iter().map(|t| t.size).sum();

    println!(
        "Found {} CUDA packages ({})",
        cuda_tasks.len(),
        format_size(cuda_total_size)
    );

    // Find compatible cuDNN
    println!("Finding compatible cuDNN version...");
    let cudnn_task =
        if let Some((cudnn_version, cuda_variant)) = find_compatible_cudnn(version).await? {
            println!(
                "Found compatible cuDNN {} ({})",
                cudnn_version, cuda_variant
            );

            let cudnn_metadata = fetch_cudnn_version_metadata(&cudnn_version).await?;
            collect_cudnn_download_task(&cudnn_metadata, &cuda_variant)?
        } else {
            println!("Warning: No compatible cuDNN found for CUDA {}", version);
            None
        };

    let cudnn_size = cudnn_task.as_ref().map(|t| t.size).unwrap_or(0);
    let total_size = cuda_total_size + cudnn_size;

    println!("\nTotal download size: {}", format_size(total_size));
    println!("Installation directory: {}\n", install_dir.display());

    // Create directories
    let downloads = downloads_dir()?;
    fs::create_dir_all(&downloads).await?;
    fs::create_dir_all(&install_dir).await?;

    let client = Client::new();

    // Download and extract CUDA packages
    for (i, task) in cuda_tasks.iter().enumerate() {
        println!(
            "[{}/{}] Downloading {} ({})...",
            i + 1,
            cuda_tasks.len() + cudnn_task.iter().count(),
            task.package_name,
            format_size(task.size)
        );

        let archive_name = task
            .relative_path
            .split('/')
            .next_back()
            .unwrap_or("archive.tar.xz");
        let archive_path = downloads.join(archive_name);

        download_file(&client, &task.url, &archive_path).await?;

        print!("  Verifying checksum...");
        if !verify_checksum(&archive_path, &task.sha256).await? {
            fs::remove_file(&archive_path).await.ok();
            bail!("Checksum verification failed for {}", task.package_name);
        }
        println!(" OK");

        print!("  Extracting...");
        extract_tarball(&archive_path, &install_dir).await?;
        println!(" OK");

        fs::remove_file(&archive_path).await.ok();
    }

    // Download and extract cuDNN
    if let Some(task) = cudnn_task {
        println!(
            "[{}/{}] Downloading cuDNN ({})...",
            cuda_tasks.len() + 1,
            cuda_tasks.len() + 1,
            format_size(task.size)
        );

        let archive_name = task
            .relative_path
            .split('/')
            .next_back()
            .unwrap_or("cudnn.tar.xz");
        let archive_path = downloads.join(archive_name);

        download_file(&client, &task.url, &archive_path).await?;

        print!("  Verifying checksum...");
        if !verify_checksum(&archive_path, &task.sha256).await? {
            fs::remove_file(&archive_path).await.ok();
            bail!("Checksum verification failed for cuDNN");
        }
        println!(" OK");

        print!("  Extracting...");
        extract_tarball(&archive_path, &install_dir).await?;
        println!(" OK");

        fs::remove_file(&archive_path).await.ok();
    }

    println!("\n✓ CUDA {} installed successfully!", version);
    println!("\nTo use this version, run:");
    println!("  cudup use {}", version);

    Ok(())
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
    fn test_format_size_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn test_format_size_kilobytes() {
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(2048), "2.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
    }

    #[test]
    fn test_format_size_megabytes() {
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 100), "100.00 MB");
        assert_eq!(format_size(1024 * 1024 + 512 * 1024), "1.50 MB");
    }

    #[test]
    fn test_format_size_gigabytes() {
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_size(1024 * 1024 * 1024 * 5), "5.00 GB");
    }

    #[test]
    fn test_collect_cuda_download_tasks() {
        let metadata = sample_cuda_metadata();
        let tasks = collect_cuda_download_tasks(&metadata, "12.4.1").unwrap();

        // Should have 2 packages (cuda_cccl and cuda_cudart), release_label is skipped
        assert_eq!(tasks.len(), 2);

        let cccl_task = tasks.iter().find(|t| t.package_name == "cuda_cccl");
        assert!(cccl_task.is_some());
        let cccl = cccl_task.unwrap();
        assert_eq!(cccl.size, 1234567);
        assert!(cccl.url.contains("cuda_cccl-linux-x86_64"));
        assert!(!cccl.sha256.is_empty());
    }

    #[test]
    fn test_collect_cuda_download_tasks_skips_release_packages() {
        let metadata = sample_cuda_metadata();
        let tasks = collect_cuda_download_tasks(&metadata, "12.4.1").unwrap();

        // release_label package should be skipped
        let release_task = tasks
            .iter()
            .find(|t| t.package_name.starts_with("release_"));
        assert!(release_task.is_none());
    }

    #[test]
    fn test_collect_cudnn_download_task_cuda12() {
        let metadata = sample_cudnn_metadata();
        let task = collect_cudnn_download_task(&metadata, "cuda12").unwrap();

        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.package_name, "cudnn");
        assert_eq!(task.size, 987654322);
        assert!(task.url.contains("cuda12-archive"));
        assert!(task.relative_path.contains("cuda12"));
    }

    #[test]
    fn test_collect_cudnn_download_task_cuda11() {
        let metadata = sample_cudnn_metadata();
        let task = collect_cudnn_download_task(&metadata, "cuda11").unwrap();

        assert!(task.is_some());
        let task = task.unwrap();
        assert!(task.url.contains("cuda11-archive"));
    }

    #[test]
    fn test_collect_cudnn_download_task_invalid_variant() {
        let metadata = sample_cudnn_metadata();
        let task = collect_cudnn_download_task(&metadata, "cuda10").unwrap();

        // CUDA 10 is not supported
        assert!(task.is_none());
    }

    #[test]
    fn test_collect_cudnn_download_task_no_cudnn_package() {
        let metadata = sample_cuda_metadata(); // CUDA metadata, no cuDNN
        let task = collect_cudnn_download_task(&metadata, "cuda12").unwrap();

        assert!(task.is_none());
    }

    #[test]
    fn test_version_install_dir() {
        let dir = version_install_dir("12.4.1").unwrap();
        assert!(dir.to_string_lossy().contains("12.4.1"));
        assert!(dir.to_string_lossy().contains(".cudup"));
        assert!(dir.to_string_lossy().contains("versions"));
    }

    #[test]
    fn test_downloads_dir() {
        let dir = downloads_dir().unwrap();
        assert!(dir.to_string_lossy().contains(".cudup"));
        assert!(dir.to_string_lossy().contains("downloads"));
    }

    #[tokio::test]
    async fn test_verify_checksum_correct() {
        use tokio::io::AsyncWriteExt;

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");

        // Create a test file with known content
        let content = b"Hello, World!";
        let mut file = tokio::fs::File::create(&file_path).await.unwrap();
        file.write_all(content).await.unwrap();
        file.flush().await.unwrap();

        // SHA256 of "Hello, World!" is known
        let expected_sha256 = "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f";

        let result = verify_checksum(&file_path, expected_sha256).await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_verify_checksum_incorrect() {
        use tokio::io::AsyncWriteExt;

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");

        let content = b"Hello, World!";
        let mut file = tokio::fs::File::create(&file_path).await.unwrap();
        file.write_all(content).await.unwrap();
        file.flush().await.unwrap();

        let wrong_sha256 = "0000000000000000000000000000000000000000000000000000000000000000";

        let result = verify_checksum(&file_path, wrong_sha256).await.unwrap();
        assert!(!result);
    }

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
