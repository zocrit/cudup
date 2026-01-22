use anyhow::Result;
use std::fs;
use std::io::Write;

use crate::config::cudup_home;

use super::{Shell, env_file_path, is_rc_configured, prompt_confirmation};

pub fn setup() -> Result<()> {
    let shell = Shell::detect()?;
    let env_path = env_file_path(&shell)?;
    let rc_path = shell.rc_file()?;

    println!("Detected shell: {}", shell.name());
    println!();

    let rc_configured = is_rc_configured(&rc_path)?;
    let env_exists = env_path.exists();

    match (rc_configured, env_exists) {
        (true, true) => {
            println!("cudup is already configured:");
            println!("  - {}", env_path.display());
            println!("  - {} (contains source line)", rc_path.display());
            println!();

            if !prompt_confirmation("Reconfigure anyway?")? {
                println!("No changes made.");
                return Ok(());
            }
            println!();
        }
        (true, false) => {
            println!(
                "Note: {} references cudup but {} is missing.",
                rc_path.display(),
                env_path.display()
            );
            println!("This will recreate the env file.");
            println!();
        }
        (false, true) => {
            println!(
                "Note: {} exists but {} doesn't source it.",
                env_path.display(),
                rc_path.display()
            );
            println!("This will update both files.");
            println!();
        }
        (false, false) => {}
    }

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

    if !prompt_confirmation("Proceed with setup?")? {
        println!("Setup cancelled.");
        return Ok(());
    }

    fs::create_dir_all(cudup_home()?)?;

    fs::write(&env_path, shell.env_content())?;
    println!();
    println!("Created {}", env_path.display());

    if !rc_configured {
        let mut rc_file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&rc_path)?;

        writeln!(rc_file)?;
        writeln!(rc_file, "# cudup")?;
        writeln!(rc_file, "{}", shell.source_line())?;
        println!("Updated {}", rc_path.display());
    }

    println!();
    println!("Setup complete!");
    if !rc_configured {
        println!();
        println!("To start using cudup, either:");
        println!("  - Restart your terminal, or");
        println!("  - Run: source {}", rc_path.display());
    }

    Ok(())
}
