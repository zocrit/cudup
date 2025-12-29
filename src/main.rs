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
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Install { version } => commands::install(version),
    }
}
