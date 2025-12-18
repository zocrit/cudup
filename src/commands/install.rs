use crate::cuda::CudaVersion;
use crate::fetch;
use anyhow::Result;

pub async fn install(version: &CudaVersion) -> Result<()> {
    fetch::install_cuda_version(version).await
}
