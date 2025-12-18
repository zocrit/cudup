mod remove;
mod setup;

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::{env, fs};

use crate::config::cudup_home;
pub use crate::config::prompt_confirmation;

pub use remove::remove;
pub use setup::setup;

const BASH_ZSH_ENV: &str = r#"# cudup shell integration
cudup() {
    if [[ "$1" == "use" ]]; then
        eval "$(command cudup use "${@:2}")"
    else
        command cudup "$@"
    fi
}
"#;

const FISH_ENV: &str = r#"# cudup shell integration
function cudup
    if test (count $argv) -gt 0 && test "$argv[1]" = "use"
        eval (command cudup use $argv[2..])
    else
        command cudup $argv
    end
end
"#;

const CUDUP_COMMENT: &str = "# cudup";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

impl Shell {
    pub fn detect() -> Result<Self> {
        let shell_path = env::var("SHELL").context("Could not detect shell from $SHELL")?;
        let shell_name = Path::new(&shell_path)
            .file_name()
            .and_then(|n| n.to_str())
            .context("Could not determine shell name from $SHELL")?;

        match shell_name {
            "fish" => Ok(Shell::Fish),
            "zsh" => Ok(Shell::Zsh),
            "bash" => Ok(Shell::Bash),
            _ => bail!(
                "Unsupported shell: {}. Supported shells: bash, zsh, fish.",
                shell_path
            ),
        }
    }

    pub fn env_content(&self) -> &'static str {
        match self {
            Shell::Bash | Shell::Zsh => BASH_ZSH_ENV,
            Shell::Fish => FISH_ENV,
        }
    }

    pub fn rc_file(&self) -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(match self {
            Shell::Bash => home.join(".bashrc"),
            Shell::Zsh => home.join(".zshrc"),
            Shell::Fish => home.join(".config/fish/config.fish"),
        })
    }

    pub fn source_line(&self) -> String {
        let env_file = self.env_file_name();
        match self {
            Shell::Bash | Shell::Zsh => format!(r#". "$HOME/.cudup/{}""#, env_file),
            Shell::Fish => format!(r#"source "$HOME/.cudup/{}""#, env_file),
        }
    }

    pub fn env_file_name(&self) -> &'static str {
        match self {
            Shell::Bash | Shell::Zsh => "env",
            Shell::Fish => "env.fish",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
        }
    }
}

pub struct ManageContext {
    pub shell: Shell,
    pub env_path: PathBuf,
    pub rc_path: PathBuf,
    pub rc_configured: bool,
    pub env_exists: bool,
}

impl ManageContext {
    pub fn detect() -> Result<Self> {
        let shell = Shell::detect()?;
        let env_path = env_file_path(shell)?;
        let rc_path = shell.rc_file()?;
        let rc_configured = is_rc_configured(&rc_path)?;
        let env_exists = env_path.exists();

        Ok(Self {
            shell,
            env_path,
            rc_path,
            rc_configured,
            env_exists,
        })
    }

    pub fn print_detected_shell(&self) {
        println!("Detected shell: {}", self.shell.name());
        println!();
    }
}

pub fn env_file_path(shell: Shell) -> Result<PathBuf> {
    Ok(cudup_home()?.join(shell.env_file_name()))
}

pub fn is_rc_configured(rc_path: &Path) -> Result<bool> {
    if !rc_path.exists() {
        return Ok(false);
    }
    let content = fs::read_to_string(rc_path)?;
    Ok(content.lines().any(|line| {
        let trimmed = line.trim();
        (trimmed.starts_with(". ") || trimmed.starts_with("source "))
            && trimmed.contains(".cudup/env")
    }))
}

pub fn remove_cudup_lines(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if line.trim() == CUDUP_COMMENT {
            i += 1;
            if i < lines.len() && lines[i].contains(".cudup/env") {
                i += 1;
            }
            if result.last().is_some_and(|s: &&str| s.is_empty()) {
                result.pop();
            }
            continue;
        }

        if line.contains(".cudup/env") {
            i += 1;
            continue;
        }

        result.push(line);
        i += 1;
    }

    while result.last().is_some_and(|s: &&str| s.is_empty()) {
        result.pop();
    }

    if result.is_empty() {
        String::new()
    } else {
        result.join("\n") + "\n"
    }
}
