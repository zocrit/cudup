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

use super::download::{DownloadTask, download_file};
use super::extract::extract_tarball;
use super::tasks::{
    collect_cuda_download_tasks, collect_cudnn_download_task, find_compatible_cudnn,
};
use super::utils::{format_size, version_install_dir};
use super::verify::verify_checksum;
use crate::config;

fn create_progress_bar(mp: &MultiProgress, size: u64, prefix: String) -> ProgressBar {
    let pb = mp.add(ProgressBar::new(size));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{prefix:>12.green.bold} [{bar:30.green/dim}] {bytes:>10}/{total_bytes:<10} {bytes_per_sec:>12} ({eta})")
            .expect("invalid progress bar template")
            .progress_chars("━━╸"),
    );
    pb.set_prefix(prefix);
    pb
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

async fn process_download_task(
    client: &Client,
    task: &DownloadTask,
    downloads_dir: &Path,
    install_dir: &Path,
    mp: &MultiProgress,
) -> Result<()> {
    let archive_path = downloads_dir.join(task.archive_name());

    // Download with progress bar
    let pb = create_progress_bar(mp, task.size, task.package_name.clone());
    download_file(client, &task.url, &archive_path, Some(&pb)).await?;
    pb.finish_with_message("downloaded");

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

pub async fn install_cuda_version(version: &str) -> Result<()> {
    let mp = MultiProgress::new();

    // Check version availability
    let check_spinner = create_spinner(&mp, "Checking available versions...".to_string());
    let available_versions = fetch_available_cuda_versions().await?;
    check_spinner.finish_and_clear();

    if !available_versions.contains(version) {
        bail!(
            "CUDA version {} is not available. Use 'cudup list' to see available versions.",
            version
        );
    }
    info!("Version {} available", version);

    let install_dir = version_install_dir(version)?;
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
    let cuda_metadata = fetch_cuda_version_metadata(version).await?;
    let cuda_tasks = collect_cuda_download_tasks(&cuda_metadata, version);
    let cuda_total_size: u64 = cuda_tasks.iter().map(|t| t.size).sum();
    meta_spinner.finish_and_clear();
    info!(
        "Found {} CUDA packages ({})",
        cuda_tasks.len(),
        format_size(cuda_total_size)
    );

    // Find compatible cuDNN
    let cudnn_spinner = create_spinner(&mp, "Finding compatible cuDNN version...".to_string());
    let cudnn_result = find_compatible_cudnn(version).await?;
    cudnn_spinner.finish_and_clear();

    let cudnn_task = match cudnn_result {
        Some((cudnn_version, cuda_variant)) => {
            info!("Found cuDNN {} ({})", cudnn_version, cuda_variant);
            let cudnn_metadata = fetch_cudnn_version_metadata(&cudnn_version).await?;
            collect_cudnn_download_task(&cudnn_metadata, &cuda_variant)
        }
        None => {
            warn!("No compatible cuDNN found for CUDA {}", version);
            None
        }
    };

    let cudnn_size = cudnn_task.as_ref().map_or(0, |t| t.size);
    let total_size = cuda_total_size + cudnn_size;
    let total_packages = cuda_tasks.len() + usize::from(cudnn_task.is_some());

    info!(
        "Downloading {} packages ({})",
        total_packages,
        format_size(total_size)
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
