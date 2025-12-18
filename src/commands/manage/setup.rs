use anyhow::Result;
use std::fs;
use std::io::Write;

use crate::config::cudup_home;

use super::{ManageContext, prompt_confirmation};

pub fn setup() -> Result<()> {
    let ctx = ManageContext::detect()?;
    ctx.print_detected_shell();

    let ManageContext {
        shell,
        env_path,
        rc_path,
        rc_configured,
        env_exists,
    } = ctx;

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
                "{} references cudup but {} is missing.",
                rc_path.display(),
                env_path.display()
            );
            println!("This will recreate the env file.");
            println!();
        }
        (false, true) => {
            println!(
                "{} exists but {} doesn't source it.",
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
