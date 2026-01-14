use anyhow::Result;
use std::fs;

use super::{Shell, env_file_path, is_rc_configured, prompt_confirmation, remove_cudup_lines};

pub fn remove() -> Result<()> {
    let shell = Shell::detect()?;
    let env_path = env_file_path()?;
    let rc_path = shell.rc_file()?;

    println!("Detected shell: {}\n", shell.name());

    let rc_configured = is_rc_configured(&rc_path)?;
    let env_exists = env_path.exists();

    if !rc_configured && !env_exists {
        println!("cudup is not configured. Nothing to remove.");
        return Ok(());
    }

    // Show what will be removed
    println!("This will:");
    if env_exists {
        println!("  - Delete: {}", env_path.display());
    }
    if rc_configured {
        println!("  - Remove cudup lines from: {}", rc_path.display());
    }
    println!();

    // Ask for confirmation
    if !prompt_confirmation("Proceed with removal?")? {
        println!("Removal cancelled.");
        return Ok(());
    }

    // Remove env file
    if env_exists {
        fs::remove_file(&env_path)?;
        println!("\nDeleted {}", env_path.display());
    }

    // Remove cudup lines from rc file
    if rc_configured {
        let content = fs::read_to_string(&rc_path)?;
        let new_content = remove_cudup_lines(&content);
        fs::write(&rc_path, new_content)?;
        println!("Updated {}", rc_path.display());
    }

    println!("\nRemoval complete!");
    println!("\nTo apply changes, either:");
    println!("  - Restart your terminal, or");
    println!("  - Run: source {}", rc_path.display());

    Ok(())
}
