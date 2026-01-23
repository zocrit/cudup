use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::Write;

mod commands;
mod config;
mod cuda;
mod fetch;

use cuda::CudaVersion;

#[derive(Parser)]
#[command(name = "cudup", author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install {
        #[arg(
            help = "CUDA version to install (e.g., 12.4.1)",
            value_name = "VERSION",
            value_parser = clap::value_parser!(CudaVersion)
        )]
        version: CudaVersion,
    },
    Uninstall {
        #[arg(
            help = "CUDA version to uninstall (e.g., 12.4.1)",
            value_name = "VERSION",
            required_unless_present = "all",
            value_parser = clap::value_parser!(CudaVersion)
        )]
        version: Option<CudaVersion>,
        #[arg(short, long, help = "Skip confirmation prompt")]
        force: bool,
        #[arg(short, long, help = "Uninstall all versions")]
        all: bool,
    },
    List,
    Check,
    Use {
        #[arg(
            help = "CUDA version to activate (e.g., 12.4.1)",
            value_name = "VERSION",
            value_parser = clap::value_parser!(CudaVersion)
        )]
        version: CudaVersion,
    },
    Manage {
        #[command(subcommand)]
        command: ManageCommand,
    },
}

#[derive(Subcommand)]
enum ManageCommand {
    Setup,
    Remove,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let level_style = buf.default_level_style(record.level());
            writeln!(
                buf,
                "{level_style}{}{level_style:#} {}",
                record.level(),
                record.args()
            )
        })
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Install { version } => commands::install(version).await?,
        Commands::Uninstall {
            version,
            force,
            all,
        } => commands::uninstall(version.as_ref().map(CudaVersion::as_str), *force, *all)?,
        Commands::List => commands::list_available_versions().await?,
        Commands::Check => commands::check()?,
        Commands::Use { version } => commands::use_version(version.as_str())?,
        Commands::Manage { command } => match command {
            ManageCommand::Setup => commands::setup()?,
            ManageCommand::Remove => commands::remove()?,
        },
    }

    Ok(())
}
