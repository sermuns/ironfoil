use clap::{Parser, Subcommand};
use ironfoil_core::{perform_tinfoil_network_install, perform_tinfoil_usb_install};
use std::{net::Ipv4Addr, path::PathBuf};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to a game backup file or directory containing game backup files
    game_backup_path: PathBuf,

    /// Whether to recursively look for files. Only applicable when pointing to a directory
    #[arg(short, long)]
    recurse: bool,

    #[command(subcommand)]
    transfer_type: TransferType,
}

#[derive(Debug, Subcommand)]
enum TransferType {
    /// Transfer over USB
    Usb,

    /// Transfer over network
    #[command(arg_required_else_help = true)]
    Network {
        /// The IP address of the Nintendo Switch
        target_ip: Ipv4Addr,
    },
}

fn main() -> color_eyre::Result<()> {
    env_logger::builder().format_source_path(true).init();
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .display_location_section(cfg!(debug_assertions))
        .install()?;

    let args = Cli::parse();

    match args.transfer_type {
        TransferType::Usb => perform_tinfoil_usb_install(&args.game_backup_path, args.recurse),
        TransferType::Network { target_ip } => {
            perform_tinfoil_network_install(&args.game_backup_path, target_ip, args.recurse)
        }
    }
}
