use anyhow::{Result, bail};
use log::info;

use crate::install;

/// Generates shell commands to activate a specific CUDA version
pub async fn use_version(version: &str) -> Result<()> {
    // Check if the version is installed locally
    let install_dir = install::version_install_dir(version)?;
    if !install_dir.exists() {
        bail!(
            "CUDA {} is not installed.\n\
             Run 'cudup install {}' to install it.",
            version,
            version
        );
    }

    // Generate shell export commands (eval'd by shell function from `cudup setup`)
    let cuda_home = install_dir.display();
    println!("export CUDA_HOME=\"{}\"", cuda_home);
    println!("export PATH=\"$CUDA_HOME/bin${{PATH:+:$PATH}}\"");
    println!("export LD_LIBRARY_PATH=\"$CUDA_HOME/lib64${{LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}}\"");

    info!("CUDA {} activated", version);

    Ok(())
}
