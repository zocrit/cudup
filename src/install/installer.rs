use anyhow::{Result, bail};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Client;
use std::path::Path;
use tokio::fs;

use crate::cuda::discover::{
    fetch_available_cuda_versions, fetch_cuda_version_metadata, fetch_cudnn_version_metadata,
};

use super::download::{DownloadTask, download_file};
use super::extract::extract_tarball;
use super::tasks::{
    collect_cuda_download_tasks, collect_cudnn_download_task, find_compatible_cudnn,
};
use super::utils::{downloads_dir, format_size, version_install_dir};
use super::verify::verify_checksum;

/// Creates a progress bar with consistent styling
fn create_progress_bar(mp: &MultiProgress, size: u64, prefix: String) -> ProgressBar {
    let pb = mp.add(ProgressBar::new(size));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{prefix:.cyan.bold} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("█▓░"),
    );
    pb.set_prefix(prefix);
    pb
}

/// Creates a spinner for operations without known size
fn create_spinner(mp: &MultiProgress, message: String) -> ProgressBar {
    let spinner = mp.add(ProgressBar::new_spinner());
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message(message);
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    spinner
}

/// Downloads, verifies, and extracts a single task
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
    if !verify_checksum(&archive_path, &task.sha256).await? {
        verify_spinner.finish_with_message("✗ checksum failed");
        fs::remove_file(&archive_path).await.ok();
        bail!("Checksum verification failed for {}", task.package_name);
    }
    verify_spinner.finish_with_message(format!("✓ {} verified", task.package_name));

    // Extract
    let extract_spinner = create_spinner(mp, format!("Extracting {}...", task.package_name));
    extract_tarball(&archive_path, install_dir).await?;
    extract_spinner.finish_with_message(format!("✓ {} extracted", task.package_name));

    // Cleanup
    fs::remove_file(&archive_path).await.ok();

    Ok(())
}

pub async fn install_cuda_version(version: &str) -> Result<()> {
    let mp = MultiProgress::new();

    // Check version availability
    let check_spinner = create_spinner(&mp, "Checking available versions...".to_string());
    let available_versions = fetch_available_cuda_versions().await?;
    if !available_versions.contains(version) {
        check_spinner.finish_with_message("✗ version not found");
        bail!(
            "CUDA version {} is not available. Use 'cudup list' to see available versions.",
            version
        );
    }
    check_spinner.finish_with_message("✓ Version available");

    let install_dir = version_install_dir(version)?;
    if install_dir.exists() {
        bail!(
            "CUDA {} is already installed at {}",
            version,
            install_dir.display()
        );
    }

    println!(
        "\n📦 Installing CUDA {} to {}\n",
        version,
        install_dir.display()
    );

    // Fetch CUDA metadata
    let meta_spinner = create_spinner(&mp, format!("Fetching CUDA {} metadata...", version));
    let cuda_metadata = fetch_cuda_version_metadata(version).await?;
    let cuda_tasks = collect_cuda_download_tasks(&cuda_metadata, version)?;
    let cuda_total_size: u64 = cuda_tasks.iter().map(|t| t.size).sum();
    meta_spinner.finish_with_message(format!(
        "✓ Found {} CUDA packages ({})",
        cuda_tasks.len(),
        format_size(cuda_total_size)
    ));

    // Find compatible cuDNN
    let cudnn_spinner = create_spinner(&mp, "Finding compatible cuDNN version...".to_string());
    let cudnn_task =
        if let Some((cudnn_version, cuda_variant)) = find_compatible_cudnn(version).await? {
            cudnn_spinner.finish_with_message(format!(
                "✓ Found cuDNN {} ({})",
                cudnn_version, cuda_variant
            ));

            let cudnn_metadata = fetch_cudnn_version_metadata(&cudnn_version).await?;
            collect_cudnn_download_task(&cudnn_metadata, &cuda_variant)?
        } else {
            cudnn_spinner.finish_with_message("⚠ No compatible cuDNN found");
            None
        };

    let cudnn_size = cudnn_task.as_ref().map(|t| t.size).unwrap_or(0);
    let total_size = cuda_total_size + cudnn_size;
    let total_packages = cuda_tasks.len() + cudnn_task.iter().count();

    println!(
        "\n📥 Downloading {} packages ({})\n",
        total_packages,
        format_size(total_size)
    );

    // Create directories
    let downloads = downloads_dir()?;
    fs::create_dir_all(&downloads).await?;
    fs::create_dir_all(&install_dir).await?;

    let client = Client::new();

    // Process all CUDA packages
    for task in &cuda_tasks {
        process_download_task(&client, task, &downloads, &install_dir, &mp).await?;
    }

    // Process cuDNN if available
    if let Some(task) = &cudnn_task {
        process_download_task(&client, task, &downloads, &install_dir, &mp).await?;
    }

    println!("\n✅ CUDA {} installed successfully!\n", version);
    println!("To use this version, run:");
    println!("  cudup use {}\n", version);

    Ok(())
}
