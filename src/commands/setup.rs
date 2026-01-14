use anyhow::{Context, Result, bail};
use std::io::{self, Write};
use std::path::PathBuf;
use std::{env, fs};

use crate::config::cudup_home;

const BASH_ENV: &str = r#"# cudup shell integration
cudup() {
    if [[ "$1" == "use" ]]; then
        eval "$(command cudup use "${@:2}")"
    else
        command cudup "$@"
    fi
}
"#;

const ZSH_ENV: &str = r#"# cudup shell integration
cudup() {
    if [[ "$1" == "use" ]]; then
        eval "$(command cudup use "${@:2}")"
    else
        command cudup "$@"
    fi
}
"#;

const SOURCE_LINE: &str = r#". "$HOME/.cudup/env""#;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Shell {
    Bash,
    Zsh,
}

impl Shell {
    fn detect() -> Result<Self> {
        let shell = env::var("SHELL").context("Could not detect shell from $SHELL")?;
        if shell.contains("zsh") {
            Ok(Shell::Zsh)
        } else if shell.contains("bash") {
            Ok(Shell::Bash)
        } else {
            bail!("Unsupported shell: {}. Only bash and zsh are supported.", shell)
        }
    }

    fn env_content(&self) -> &'static str {
        match self {
            Shell::Bash => BASH_ENV,
            Shell::Zsh => ZSH_ENV,
        }
    }

    fn rc_file(&self) -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(match self {
            Shell::Bash => home.join(".bashrc"),
            Shell::Zsh => home.join(".zshrc"),
        })
    }

    fn name(&self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
        }
    }
}

fn env_file_path() -> Result<PathBuf> {
    Ok(cudup_home()?.join("env"))
}

fn is_already_configured(rc_path: &PathBuf) -> Result<bool> {
    if !rc_path.exists() {
        return Ok(false);
    }
    let content = fs::read_to_string(rc_path)?;
    Ok(content.contains(".cudup/env"))
}

fn prompt_confirmation(message: &str) -> Result<bool> {
    print!("{} [y/N] ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}

pub fn setup() -> Result<()> {
    let shell = Shell::detect()?;
    let env_path = env_file_path()?;
    let rc_path = shell.rc_file()?;

    println!("Detected shell: {}\n", shell.name());

    // Check if already configured
    let rc_configured = is_already_configured(&rc_path)?;
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
        println!("Note: {} references cudup but {} is missing.", rc_path.display(), env_path.display());
        println!("This will recreate the env file.\n");
    } else if env_exists {
        println!("Note: {} exists but {} doesn't source it.", env_path.display(), rc_path.display());
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
