mod remove;
mod setup;

use anyhow::{Context, Result, bail};
use std::io::{self, Write};
use std::path::PathBuf;
use std::{env, fs};

use crate::config::cudup_home;

pub use remove::remove;
pub use setup::setup;

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
const CUDUP_COMMENT: &str = "# cudup";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shell {
    Bash,
    Zsh,
}

impl Shell {
    pub fn detect() -> Result<Self> {
        let shell = env::var("SHELL").context("Could not detect shell from $SHELL")?;
        if shell.contains("zsh") {
            Ok(Shell::Zsh)
        } else if shell.contains("bash") {
            Ok(Shell::Bash)
        } else {
            bail!("Unsupported shell: {}. Only bash and zsh are supported.", shell)
        }
    }

    pub fn env_content(&self) -> &'static str {
        match self {
            Shell::Bash => BASH_ENV,
            Shell::Zsh => ZSH_ENV,
        }
    }

    pub fn rc_file(&self) -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(match self {
            Shell::Bash => home.join(".bashrc"),
            Shell::Zsh => home.join(".zshrc"),
        })
    }

    pub fn name(&self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
        }
    }
}

pub fn env_file_path() -> Result<PathBuf> {
    Ok(cudup_home()?.join("env"))
}

pub fn is_rc_configured(rc_path: &PathBuf) -> Result<bool> {
    if !rc_path.exists() {
        return Ok(false);
    }
    let content = fs::read_to_string(rc_path)?;
    Ok(content.contains(".cudup/env"))
}

pub fn prompt_confirmation(message: &str) -> Result<bool> {
    print!("{} [y/N] ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}

/// Removes cudup-related lines from the rc file content
pub fn remove_cudup_lines(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Skip the "# cudup" comment and the following source line
        if line.trim() == CUDUP_COMMENT {
            // Skip this line
            i += 1;
            // Skip the next line if it's the source line
            if i < lines.len() && lines[i].contains(".cudup/env") {
                i += 1;
            }
            // Skip any blank line that preceded the comment (remove trailing blank)
            if !result.is_empty() && result.last().map(|s: &&str| s.is_empty()).unwrap_or(false) {
                result.pop();
            }
            continue;
        }

        // Skip standalone source lines (without comment)
        if line.contains(".cudup/env") {
            i += 1;
            continue;
        }

        result.push(line);
        i += 1;
    }

    // Remove trailing empty lines
    while result.last().map(|s| s.is_empty()).unwrap_or(false) {
        result.pop();
    }

    if result.is_empty() {
        String::new()
    } else {
        result.join("\n") + "\n"
    }
}
