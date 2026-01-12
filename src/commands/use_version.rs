use anyhow::{Result, bail};

use crate::install;

/// Generates shell commands to activate a specific CUDA version
pub async fn use_version(version: &Option<String>) -> Result<()> {
    let version = match version {
        Some(v) => v.clone(),
        None => {
            bail!(
                "Please specify a CUDA version to use.\n\
                 Usage: cudup use <version>\n\
                 Example: cudup use 12.3.1\n\n\
                 Run 'cudup list' to see available versions."
            );
        }
    };

    // Check if the version is installed locally
    let install_dir = install::version_install_dir(&version)?;
    if !install_dir.exists() {
        bail!(
            "CUDA {} is not installed.\n\
             Run 'cudup install {}' to install it, or 'cudup list' to see available versions.",
            version,
            version
        );
    }

    // Generate shell export commands
    let cuda_home = install_dir.display();

    // Print shell commands to stdout (to be eval'd)
    // Using ${VAR:+:$VAR} syntax to handle empty variables gracefully
    println!("export CUDA_HOME=\"{}\"", cuda_home);
    println!("export PATH=\"$CUDA_HOME/bin${{PATH:+:$PATH}}\"");
    println!("export LD_LIBRARY_PATH=\"$CUDA_HOME/lib64${{LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}}\"");

    // Print usage instructions to stderr (so they don't interfere with eval)
    eprintln!();
    eprintln!("# CUDA {} activated", version);
    eprintln!("# ");
    eprintln!("# To use this version, run:");
    eprintln!("#   eval \"$(cudup use {})\"", version);
    eprintln!("# ");
    eprintln!("# Or add to your shell config (~/.bashrc or ~/.zshrc):");
    eprintln!("#   eval \"$(cudup use {})\"", version);

    Ok(())
}
