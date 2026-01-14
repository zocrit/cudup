use crate::install;
use anyhow::Result;

pub async fn install(version: &str) -> Result<()> {
    install::install_cuda_version(version).await
}
