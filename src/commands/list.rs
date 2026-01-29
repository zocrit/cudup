use anyhow::{Context, Result};

use crate::{config, cuda::discover::fetch_available_cuda_versions};

pub async fn list_available_versions() -> Result<()> {
    let versions = fetch_available_cuda_versions()
        .await
        .context("Failed to fetch available CUDA versions")?;

    if versions.is_empty() {
        println!("No CUDA versions available");
        return Ok(());
    }

    let versions_dir = config::versions_dir().ok();

    println!("Available CUDA versions:");
    for version in &versions {
        let installed = versions_dir
            .as_ref()
            .is_some_and(|dir| dir.join(version).exists());
        println!("{} {:>10}", if installed { "*" } else { " " }, version);
    }

    println!();
    println!("* = installed");

    Ok(())
}
