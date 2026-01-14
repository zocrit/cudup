use anyhow::Result;
use std::fs;
use std::io::Write;

use crate::config::cudup_home;

use super::{env_file_path, is_rc_configured, prompt_confirmation, Shell, SOURCE_LINE};

pub fn setup() -> Result<()> {
    let shell = Shell::detect()?;
    let env_path = env_file_path()?;
    let rc_path = shell.rc_file()?;

    println!("Detected shell: {}\n", shell.name());

    // Check if already configured
    let rc_configured = is_rc_configured(&rc_path)?;
    let env_exists = env_path.exists();

    if rc_configured && env_exists {
        println!("cudup is already configured:");
        println!("  - {}", env_path.display());
        println!("  - {} (contains source line)", rc_path.display());
        println!();

        if !prompt_confirmation("Reconfigure anyway?")? {
            println!("No changes made.");
            return Ok(());
        }
        println!();
    } else if rc_configured {
        println!(
            "Note: {} references cudup but {} is missing.",
            rc_path.display(),
            env_path.display()
        );
        println!("This will recreate the env file.\n");
    } else if env_exists {
        println!(
            "Note: {} exists but {} doesn't source it.",
            env_path.display(),
            rc_path.display()
        );
        println!("This will update both files.\n");
    }

    // Show what will be modified
    println!("This will:");
    if env_exists {
        println!("  - Overwrite: {}", env_path.display());
    } else {
        println!("  - Create: {}", env_path.display());
    }
    if !rc_configured {
        if rc_path.exists() {
            println!("  - Append to: {}", rc_path.display());
        } else {
            println!("  - Create: {}", rc_path.display());
        }
    }
    println!();

    // Ask for confirmation
    if !prompt_confirmation("Proceed with setup?")? {
        println!("Setup cancelled.");
        return Ok(());
    }

    // Create cudup home directory if needed
    let cudup_home = cudup_home()?;
    if !cudup_home.exists() {
        fs::create_dir_all(&cudup_home)?;
    }

    // Write env file
    fs::write(&env_path, shell.env_content())?;
    println!("\nCreated {}", env_path.display());

    // Append source line to rc file only if not already there
    if !rc_configured {
        let mut rc_file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&rc_path)?;

        writeln!(rc_file)?;
        writeln!(rc_file, "# cudup")?;
        writeln!(rc_file, "{}", SOURCE_LINE)?;
        println!("Updated {}", rc_path.display());
    }

    println!("\nSetup complete!");
    if !rc_configured {
        println!("\nTo start using cudup, either:");
        println!("  - Restart your terminal, or");
        println!("  - Run: source {}", rc_path.display());
    }

    Ok(())
}
