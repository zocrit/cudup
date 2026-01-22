use crate::fetch;
use anyhow::Result;

pub async fn install(version: &str) -> Result<()> {
    fetch::install_cuda_version(version).await
}
