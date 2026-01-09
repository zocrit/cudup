use anyhow::Result;

use crate::cuda::discover::fetch_available_cuda_versions;

pub async fn list_available_versions() -> Result<()> {
    let versions = fetch_available_cuda_versions().await?;

    for version in versions {
        println!("{:>10}", version);
    }

    Ok(())
}
