use anyhow::{Result, bail};

use crate::fetch;

pub fn use_version(version: &str) -> Result<()> {
    let install_dir = fetch::version_install_dir(version)?;
    if !install_dir.exists() {
        bail!(
            "CUDA {} is not installed.\n\
             Run 'cudup install {}' to install it.",
            version,
            version
        );
    }

    println!("# CUDA {} activated", version);
    println!("export CUDA_HOME=\"{}\"", install_dir.display());
    println!("export PATH=\"$CUDA_HOME/bin${{PATH:+:$PATH}}\"");
    println!("export LD_LIBRARY_PATH=\"$CUDA_HOME/lib64${{LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}}\"");

    Ok(())
}
