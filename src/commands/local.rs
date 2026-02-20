use std::path::PathBuf;

use anyhow::{Result, bail};

use crate::cuda::CudaVersion;
use crate::fetch;

const VERSION_FILE_NAME: &str = ".cuda-version";

pub struct CudaVersionConfig {
    pub cuda_version: CudaVersion,
    pub cudnn_version: Option<String>,
}

pub fn parse_cuda_version_file(contents: &str) -> Result<CudaVersionConfig> {
    let mut lines = contents
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'));

    let cuda_version = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("No CUDA version found in .cuda-version file"))
        .and_then(CudaVersion::new)?;

    let mut cudnn_version = None;
    for line in lines {
        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "cudnn" => cudnn_version = Some(value.trim().to_string()),
                other => log::warn!("Unknown key '{}' in .cuda-version file, ignoring", other),
            }
        } else {
            log::warn!(
                "Unrecognized line '{}' in .cuda-version file, ignoring",
                line
            );
        }
    }

    Ok(CudaVersionConfig {
        cuda_version,
        cudnn_version,
    })
}

pub fn find_version_file() -> Result<Option<PathBuf>> {
    let mut dir = std::env::current_dir()?;
    let home = dirs::home_dir();

    loop {
        let candidate = dir.join(VERSION_FILE_NAME);
        if candidate.is_file() {
            return Ok(Some(candidate));
        }

        if home.as_deref() == Some(&dir) {
            break;
        }

        if !dir.pop() {
            break;
        }
    }

    Ok(None)
}

pub fn local_write(version: &CudaVersion) -> Result<()> {
    let path = std::env::current_dir()?.join(VERSION_FILE_NAME);
    std::fs::write(&path, format!("{version}\n"))?;
    println!("Set CUDA {} in {}", version, path.display());

    let install_dir = fetch::version_install_dir(version.as_str())?;
    if !install_dir.exists() {
        println!(
            "Warning: CUDA {} is not installed. Run `cudup install {}` to install it.",
            version, version
        );
    }

    Ok(())
}

pub fn local_activate() -> Result<()> {
    let path = find_version_file()?.ok_or_else(|| {
        anyhow::anyhow!("No .cuda-version file found. Run `cudup local <version>` to create one.")
    })?;

    let contents = std::fs::read_to_string(&path)?;
    let config = parse_cuda_version_file(&contents)?;

    let install_dir = fetch::version_install_dir(config.cuda_version.as_str())?;
    if !install_dir.exists() {
        bail!(
            "CUDA {} is not installed. Run `cudup install {}` to install it.",
            config.cuda_version,
            config.cuda_version
        );
    }

    if config.cudnn_version.is_some() {
        log::warn!(
            "cuDNN version pinning in .cuda-version is not yet supported; ignoring cudnn key"
        );
    }

    println!(
        "# CUDA {} activated (from {})",
        config.cuda_version,
        path.display()
    );
    super::print_shell_exports(&install_dir);

    Ok(())
}
