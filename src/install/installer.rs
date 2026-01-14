use anyhow::{Result, bail};
use reqwest::Client;
use tokio::fs;

use crate::cuda::discover::{
    fetch_available_cuda_versions, fetch_cuda_version_metadata, fetch_cudnn_version_metadata,
};

use super::download::download_file;
use super::extract::extract_tarball;
use super::tasks::{
    collect_cuda_download_tasks, collect_cudnn_download_task, find_compatible_cudnn,
};
use super::utils::{downloads_dir, format_size, version_install_dir};
use super::verify::verify_checksum;

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
