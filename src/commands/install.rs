use crate::install;
use anyhow::{Result, bail};

pub async fn install(version: &Option<String>) -> Result<()> {
    let version = match version {
        Some(v) => v.clone(),
        None => {
            bail!(
                "Please specify a CUDA version to install. Use 'cudup list' to see available versions."
            );
        }
    };

    install::install_cuda_version(&version).await
}
