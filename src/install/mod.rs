use anyhow::{Context, Result, bail};
use reqwest::Client;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::cache;
use crate::cuda::discover::{
    BaseDownloadUrls, fetch_available_cuda_versions, fetch_available_cudnn_versions,
    fetch_cuda_version_metadata, fetch_cudnn_version_metadata,
};
use crate::cuda::metadata::{CudaReleaseMetadata, DownloadInfo, PlatformInfo};

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

    let cudnn_versions = fetch_available_cudnn_versions().await?;

    // Iterate from newest to oldest cuDNN version
    for cudnn_version in cudnn_versions.iter().rev() {
        let metadata = match fetch_cudnn_version_metadata(cudnn_version).await {
            Ok(m) => m,
            Err(_) => continue, // Skip versions we can't fetch
        };

        // Check if this cuDNN supports our CUDA major version
        if let Some(cudnn_pkg) = metadata.get_package("cudnn")
            && let Some(variants) = &cudnn_pkg.cuda_variant
            && variants.contains(&cuda_major.to_string())
        {
            // Found a compatible version
            let cuda_variant = format!("cuda{}", cuda_major);
            return Ok(Some((cudnn_version.clone(), cuda_variant)));
        }
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

    let bytes = fs::read(path).await?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let result = hasher.finalize();
    let actual = format!("{:x}", result);

    Ok(actual == expected_sha256)
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
    Ok(cache::versions_dir()?.join(cuda_version))
}

/// Returns the downloads directory for temporary archives
pub fn downloads_dir() -> Result<PathBuf> {
    Ok(cache::cache_dir()?.join("downloads"))
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
            .last()
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
            .last()
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
