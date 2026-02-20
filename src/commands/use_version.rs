use anyhow::{Result, bail};

use crate::fetch;

pub fn use_version(version: &str) -> Result<()> {
    let install_dir = fetch::version_install_dir(version)?;
    if !install_dir.exists() {
        bail!("CUDA {} is not installed", version);
    }

    println!("# CUDA {} activated", version);
    super::print_shell_exports(&install_dir);

    Ok(())
}
