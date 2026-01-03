use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod cuda;

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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Install { version } => commands::install(version)?,
        Commands::List {} => commands::list_available_versions()?,
    }

    Ok(())
}
