use clap::{Args, Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use ironfoil_core::{
    perform_tinfoil_network_install, perform_usb_install, read_game_paths, send_rcm_payload,
};
use std::{
    net::Ipv4Addr,
    path::{Path, PathBuf},
    sync::mpsc,
};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    install_type: InstallType,
}

#[derive(Debug, Args)]
struct InstallArgs {
    /// Path to a game backup file or directory containing game backup files
    game_backup_path: PathBuf,

    /// Whether to recursively look for files (only for directories)
    #[arg(short, long)]
    recurse: bool,
}

#[derive(Debug, Subcommand)]
enum InstallType {
    /// Transfer over USB
    #[command(arg_required_else_help = true)]
    Usb {
        #[command(flatten)]
        install_args: InstallArgs,

        /// If transferring to Sphaira homebrew menu
        #[arg(long = "sphaira")]
        for_sphaira: bool,
    },

    /// Transfer over network
    #[command(arg_required_else_help = true)]
    Network {
        #[command(flatten)]
        install_args: InstallArgs,

        /// The IP address of the Nintendo Switch
        target_ip: Ipv4Addr,
    },

    /// Inject RCM payload
    #[command(arg_required_else_help = true)]
    Rcm {
        /// Path to the RCM payload file
        payload_path: PathBuf,
    },
}

fn create_progress_bar() -> ProgressBar {
    ProgressBar::no_length().with_style(
        ProgressStyle::with_template("ETA: {eta} ({binary_bytes_per_sec}) {wide_bar} {binary_bytes} of {binary_total_bytes} sent").unwrap(),
    )
}

fn main() -> color_eyre::Result<()> {
    env_logger::builder()
        .format_source_path(cfg!(debug_assertions))
        .init();
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .display_location_section(cfg!(debug_assertions))
        .install()?;

    let args = Cli::parse();

    match args.install_type {
        InstallType::Usb {
            install_args:
                InstallArgs {
                    game_backup_path,
                    recurse,
                },
            for_sphaira,
        } => run_install(
            &game_backup_path,
            recurse,
            move |game_paths, progress_len_tx, progress_tx| {
                perform_usb_install(&game_paths, progress_len_tx, progress_tx, for_sphaira, None)
            },
        )?,
        InstallType::Network {
            install_args:
                InstallArgs {
                    game_backup_path,
                    recurse,
                },
            target_ip,
        } => run_install(
            &game_backup_path,
            recurse,
            move |game_paths, progress_len_tx, progress_tx| {
                perform_tinfoil_network_install(
                    game_paths,
                    target_ip,
                    progress_len_tx,
                    progress_tx,
                    None,
                )
            },
        )?,
        InstallType::Rcm { payload_path } => send_rcm_payload(&payload_path)?,
    }
    Ok(())
}

fn run_install<F>(
    game_backup_path: &Path,
    recurse: bool,
    install_closure: F,
) -> color_eyre::Result<()>
where
    F: FnOnce(
            Vec<std::path::PathBuf>,
            mpsc::Sender<u64>,
            mpsc::Sender<u64>,
        ) -> color_eyre::Result<()>
        + Send
        + 'static,
{
    let pb = create_progress_bar();

    let game_paths = read_game_paths(game_backup_path, recurse)?;

    let (progress_len_tx, progress_len_rx) = mpsc::channel::<u64>();
    let (progress_tx, progress_rx) = mpsc::channel::<u64>();

    let thread =
        std::thread::spawn(move || install_closure(game_paths, progress_len_tx, progress_tx));

    while !thread.is_finished() {
        if let Ok(total_len) = progress_len_rx.try_recv() {
            pb.set_length(total_len);
        }

        if let Ok(progress) = progress_rx.try_recv() {
            pb.set_position(progress);
        }
    }

    thread.join().expect("joining install thread")?;
    Ok(())
}
