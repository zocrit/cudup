use crate::cuda::discover;
use anyhow::Result;

pub fn install(version: &Option<String>) -> Result<()> {
    let versions = discover::fetch_available_cuda_versions()?;
    println!("Available CUDA versions: {:?}", versions);
    Ok(())
}
