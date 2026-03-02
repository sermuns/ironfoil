use clap::Parser;
use ns_usbloader_rs_core::perform_tinfoil_usb_install;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    game_backup_path: PathBuf,
}

fn main() -> color_eyre::Result<()> {
    env_logger::builder().format_source_path(true).init();
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .display_location_section(cfg!(debug_assertions))
        .install()?;

    let args = Args::parse();

    perform_tinfoil_usb_install(&args.game_backup_path)
}
