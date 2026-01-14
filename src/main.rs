use anyhow::Result;
use clap::{Parser, Subcommand};

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
    Install { version: Option<String> },
    List {},
    Use { version: Option<String> },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Install { version } => commands::install(version).await?,
        Commands::List {} => commands::list_available_versions().await?,
        Commands::Use { version } => commands::use_version(version).await?,
    }

    Ok(())
}
