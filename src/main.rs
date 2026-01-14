use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::Write;

mod commands;
mod config;
mod cuda;
mod install;

#[derive(Parser)]
#[command(name = "cudup", author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a specific CUDA version
    Install {
        #[arg(
            help = "CUDA version to install (e.g., 12.4.1)",
            value_name = "VERSION"
        )]
        version: String,
    },
    /// List available CUDA versions
    List,
    /// Activate a specific CUDA version
    Use {
        #[arg(
            help = "CUDA version to activate (e.g., 12.4.1)",
            value_name = "VERSION"
        )]
        version: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let level_style = buf.default_level_style(record.level());
            writeln!(buf, "{level_style}{}{level_style:#} {}", record.level(), record.args())
        })
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Install { version } => commands::install(version).await?,
        Commands::List => commands::list_available_versions().await?,
        Commands::Use { version } => commands::use_version(version).await?,
    }

    Ok(())
}
