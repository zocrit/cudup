use anyhow::Result;
use std::fs;
use std::io::ErrorKind;

use super::{ManageContext, prompt_confirmation, remove_cudup_lines};

pub fn remove() -> Result<()> {
    let ctx = ManageContext::detect()?;
    ctx.print_detected_shell();

    let ManageContext {
        env_path,
        rc_path,
        rc_configured,
        env_exists,
        ..
    } = ctx;

    if !rc_configured && !env_exists {
        println!("cudup is not configured. Nothing to remove.");
        return Ok(());
    }

    println!("This will:");
    if env_exists {
        println!("  - Delete: {}", env_path.display());
    }
    if rc_configured {
        println!("  - Remove cudup lines from: {}", rc_path.display());
    }
    println!();

    if !prompt_confirmation("Proceed with removal?")? {
        println!("Removal cancelled.");
        return Ok(());
    }

    match fs::remove_file(&env_path) {
        Ok(()) => {
            println!();
            println!("Deleted {}", env_path.display());
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            println!();
            println!("{} was already removed", env_path.display());
        }
        Err(e) => return Err(e.into()),
    }

    if rc_configured {
        let content = fs::read_to_string(&rc_path)?;
        let new_content = remove_cudup_lines(&content);
        fs::write(&rc_path, new_content)?;
        println!("Updated {}", rc_path.display());
    }

    println!();
    println!("Removal complete!");
    println!();
    println!("To apply changes, either:");
    println!("  - Restart your terminal, or");
    println!("  - Run: source {}", rc_path.display());

    Ok(())
}
