use anyhow::{Result, bail};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::{info, warn};
use reqwest::Client;
use std::path::Path;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::fs;

static DOWNLOAD_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client")
});

use crate::cuda::discover::{
    fetch_available_cuda_versions, fetch_cuda_version_metadata, fetch_cudnn_version_metadata,
};
use crate::cuda::version::CudaVersion;

use super::download::{DownloadTask, download_file};
use super::extract::extract_tarball;
use super::tasks::{
    collect_cuda_download_tasks, collect_cudnn_download_task, find_compatible_cudnn,
};
use super::utils::{format_size, target_platform, version_install_dir};
use super::verify::verify_checksum;
use crate::config;

/// Creates a progress bar for downloads. Uses a determinate bar if size is known,
/// or an indeterminate spinner showing bytes downloaded if size is unknown.
fn create_progress_bar(mp: &MultiProgress, size: Option<u64>, prefix: String) -> ProgressBar {
    match size {
        Some(s) => {
            let pb = mp.add(ProgressBar::new(s));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{prefix:>12.green.bold} [{bar:30.green/dim}] {bytes:>10}/{total_bytes:<10} {bytes_per_sec:>12} ({eta})")
                    .expect("invalid progress bar template")
                    .progress_chars("━━╸"),
            );
            pb.set_prefix(prefix);
            pb
        }
        None => {
            let pb = mp.add(ProgressBar::new_spinner());
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{prefix:>12.green.bold} {spinner} {bytes:>10} {bytes_per_sec:>12}")
                    .expect("invalid spinner template"),
            );
            pb.set_prefix(prefix);
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            pb
        }
    }
}

fn create_spinner(mp: &MultiProgress, message: String) -> ProgressBar {
    let spinner = mp.add(ProgressBar::new_spinner());
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .expect("invalid spinner template"),
    );
    spinner.set_message(message);
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    spinner
}

/// Statistics about download sizes for a set of tasks
struct SizeStats {
    known_size: u64,
    unknown_count: usize,
}

impl SizeStats {
    /// Calculate size stats from a slice of download tasks
    fn from_tasks(tasks: &[DownloadTask]) -> Self {
        Self {
            known_size: tasks.iter().filter_map(|t| t.size).sum(),
            unknown_count: tasks.iter().filter(|t| t.size.is_none()).count(),
        }
    }

    /// Combine stats from multiple sources
    fn combine(&self, other: &Self) -> Self {
        Self {
            known_size: self.known_size + other.known_size,
            unknown_count: self.unknown_count + other.unknown_count,
        }
    }

    /// Format as a human-readable size string (e.g., "5.2 GB" or "5.2 GB+")
    fn format(&self) -> String {
        if self.unknown_count > 0 {
            format!("{}+", format_size(self.known_size))
        } else {
            format_size(self.known_size)
        }
    }
}

async fn process_download_task(
    client: &Client,
    task: &DownloadTask,
    downloads_dir: &Path,
    install_dir: &Path,
    mp: &MultiProgress,
) -> Result<()> {
    let archive_path = downloads_dir.join(task.archive_name());

    let pb = create_progress_bar(mp, task.size, task.package_name.clone());
    download_file(client, &task.url, &archive_path, Some(&pb)).await?;
    pb.finish_and_clear();

    // Verify checksum
    let verify_spinner = create_spinner(mp, format!("Verifying {}...", task.package_name));
    if let Err(e) = verify_checksum(&archive_path, &task.sha256).await {
        verify_spinner
            .finish_with_message(format!("[FAIL] {} checksum mismatch", task.package_name));
        fs::remove_file(&archive_path).await.ok();
        return Err(e);
    }
    verify_spinner.finish_and_clear();

    // Extract
    let extract_spinner = create_spinner(mp, format!("Extracting {}...", task.package_name));
    extract_tarball(&archive_path, install_dir).await?;
    extract_spinner.finish_and_clear();

    // Cleanup
    fs::remove_file(&archive_path).await.ok();

    Ok(())
}

pub async fn install_cuda_version(version: &CudaVersion) -> Result<()> {
    let mp = MultiProgress::new();

    let platform = target_platform()?;
    info!("Detected platform: {}", platform);

    // Check version availability
    let check_spinner = create_spinner(&mp, "Checking available versions...".to_string());
    let available_versions = fetch_available_cuda_versions().await?;
    check_spinner.finish_and_clear();

    if !available_versions.contains(version.as_str()) {
        bail!(
            "CUDA version {} is not available. Use 'cudup list' to see available versions.",
            version
        );
    }
    info!("Version {} available", version);

    let install_dir = version_install_dir(version.as_str())?;
    if install_dir.exists() {
        bail!(
            "CUDA {} is already installed at {}",
            version,
            install_dir.display()
        );
    }

    info!("Installing CUDA {} to {}", version, install_dir.display());

    // Fetch CUDA metadata
    let meta_spinner = create_spinner(&mp, format!("Fetching CUDA {} metadata...", version));
    let cuda_metadata = fetch_cuda_version_metadata(version.as_str()).await?;
    let cuda_tasks = collect_cuda_download_tasks(&cuda_metadata, version, platform);
    meta_spinner.finish_and_clear();

    if cuda_tasks.is_empty() {
        bail!(
            "CUDA {} has no downloadable packages for platform {}. \
             This version may not support your architecture.",
            version,
            platform
        );
    }

    let cuda_stats = SizeStats::from_tasks(&cuda_tasks);
    info!("Found {} CUDA packages ({})", cuda_tasks.len(), cuda_stats.format());

    // Find compatible cuDNN
    let cudnn_spinner = create_spinner(&mp, "Finding compatible cuDNN version...".to_string());
    let cudnn_result = find_compatible_cudnn(version).await?;
    cudnn_spinner.finish_and_clear();

    let cudnn_task = match cudnn_result {
        Some((cudnn_version, cuda_variant)) => {
            info!("Found cuDNN {} ({})", cudnn_version, cuda_variant);
            let cudnn_metadata = fetch_cudnn_version_metadata(&cudnn_version).await?;
            collect_cudnn_download_task(&cudnn_metadata, &cuda_variant, platform)
        }
        None => {
            warn!("No compatible cuDNN found for CUDA {}", version);
            None
        }
    };

    let cudnn_stats = cudnn_task
        .as_ref()
        .map(|t| SizeStats::from_tasks(std::slice::from_ref(t)))
        .unwrap_or(SizeStats { known_size: 0, unknown_count: 0 });
    let total_stats = cuda_stats.combine(&cudnn_stats);
    let total_packages = cuda_tasks.len() + usize::from(cudnn_task.is_some());

    info!(
        "Downloading {} packages ({})",
        total_packages,
        total_stats.format()
    );

    // Create directories
    let downloads = config::downloads_dir()?;
    fs::create_dir_all(&downloads).await?;
    fs::create_dir_all(&install_dir).await?;

    // Download, verify, and extract all packages
    let install_result = async {
        for task in &cuda_tasks {
            process_download_task(&DOWNLOAD_CLIENT, task, &downloads, &install_dir, &mp).await?;
        }

        if let Some(task) = &cudnn_task {
            process_download_task(&DOWNLOAD_CLIENT, task, &downloads, &install_dir, &mp).await?;
        }

        Ok::<_, anyhow::Error>(())
    }
    .await;

    // Clean up partial installation on failure
    if let Err(e) = install_result {
        let _ = fs::remove_dir_all(&install_dir).await;
        return Err(e);
    }

    info!("CUDA {} installed successfully!", version);
    println!();
    println!("To use this version, run:");
    println!("  cudup use {}", version);
    println!();

    Ok(())
}
