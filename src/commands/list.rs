use anyhow::Result;

use crate::cuda::discover::fetch_available_cuda_versions;

pub fn list_available_versions() -> Result<()> {
    let versions = fetch_available_cuda_versions()?;

    for version in versions {
        println!("{:>10}", version);
    }

    Ok(())
}
