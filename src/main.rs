use clap::Parser;
use color_eyre::{
    Section,
    eyre::{ContextCompat, bail, eyre},
};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, error, info, warn};
use nusb::{
    Endpoint, MaybeFuture, list_devices,
    transfer::{Buffer, Bulk, In, Out, TransferError},
};
use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    os::unix::fs::MetadataExt,
    path::PathBuf,
    time::Duration,
};

const USB_TIMEOUT: Duration = Duration::from_millis(500);

fn write_usb(
    ep_out: &mut Endpoint<Bulk, Out>,
    message: impl Into<Vec<u8>>,
) -> color_eyre::Result<()> {
    let buf = message.into();
    ep_out
        .transfer_blocking(buf.into(), USB_TIMEOUT)
        .status
        .map_err(|e| match e {
            TransferError::Cancelled => eyre!("Nintendo Switch is not accepting transfers.")
                .suggestion("Ensure Awoo Installer is open, and in the menu 'Install Over USB'."),
            TransferError::Disconnected => eyre!("USB has disconnected"),
            TransferError::Fault | TransferError::Stall | TransferError::InvalidArgument => {
                eyre!("Malformed data during transfer.")
            }
            TransferError::Unknown(i) => eyre!("Unknown error {}", i),
        })
}

fn read_usb(ep_in: &mut Endpoint<Bulk, In>) -> Result<Buffer, TransferError> {
    // TODO: avoid creating buffer everytime?
    // TODO: figure out if 512 is universal buffer size or just my machine?
    let buf = Buffer::new(512);
    ep_in.transfer_blocking(buf, USB_TIMEOUT).into_result()
}

fn file_range_command(
    ep_in: &mut Endpoint<Bulk, In>,
    ep_out: &mut Endpoint<Bulk, Out>,
    pb: &mut ProgressBar,
    game_paths: &[String],
) -> color_eyre::Result<()> {
    let file_range_header = read_usb(ep_in)?;

    let range_size = usize::from_le_bytes(file_range_header[..8].try_into().unwrap());
    let range_offset = u64::from_le_bytes(file_range_header[8..16].try_into().unwrap());
    let game_path_len = usize::from_le_bytes(file_range_header[16..24].try_into().unwrap());

    let game_name_buf = read_usb(ep_in)?;
    let game_path = str::from_utf8(&game_name_buf)?;

    if !game_paths
        .iter()
        .any(|path| path.len() == game_path.len() + 1 && *game_path == path[..game_path.len()])
    {
        warn!("{:#?}", game_paths);
        warn!("requested: {:#?}", game_path);
        bail!("Nintendo Switch tried to request game backup not present on host");
    };

    info!("sending {}", &game_path);

    info!(
        "Range size: {}, Range offset: {}, Name len: {}, Name: {}",
        range_size, range_offset, game_path_len, game_path,
    );

    send_response_header(ep_out, range_size)?;

    let file = File::open(game_path)?;

    if let Ok(metadata) = file.metadata() {
        pb.set_length(metadata.size());
    }

    let mut reader = BufReader::new(file);

    reader.seek(SeekFrom::Start(range_offset))?;

    let mut current_offset = 0;
    let end_offset = range_size;
    let mut read_size = 0x100000;

    let mut buf = vec![0u8; read_size];

    while current_offset < end_offset {
        if current_offset + read_size >= end_offset {
            debug!("too big read_size ({}), resizing...", read_size);
            read_size = end_offset - current_offset;
            buf.resize(read_size, 0u8);
        }
        reader.read_exact(&mut buf)?;

        ep_out.transfer_blocking(buf.clone().into(), Duration::MAX);

        debug!("sent {} bytes", read_size);

        current_offset += read_size;
        pb.set_position(current_offset as u64);
    }

    Ok(())
}

fn send_response_header(
    ep_out: &mut Endpoint<Bulk, Out>,
    range_size: usize,
) -> color_eyre::Result<()> {
    write_usb(ep_out, b"TUC0")?;
    write_usb(ep_out, tinfoil_command_types::RESPONSE)?;
    write_usb(ep_out, tinfoil_command_ids::FILE_RANGE)?;
    write_usb(ep_out, range_size.to_le_bytes())?;
    write_usb(ep_out, [0u8; 0xC])?; // padding?
    Ok(())
}

mod tinfoil_command_types {
    pub const RESPONSE: [u8; 4] = 0u32.to_le_bytes();
}

mod tinfoil_command_ids {
    pub const EXIT: [u8; 4] = 0u32.to_le_bytes();
    pub const FILE_RANGE: [u8; 4] = 1u32.to_le_bytes();
}

#[derive(Parser)]
struct Args {
    game_backup_dir: PathBuf,
}

fn main() -> color_eyre::Result<()> {
    env_logger::builder().format_source_path(true).init();
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .display_location_section(cfg!(debug_assertions))
        .install()?;

    let args = Args::parse();

    if !args.game_backup_dir.exists() {
        bail!(
            "Given path ({}) does not exist",
            args.game_backup_dir.display()
        )
    }
    if !args.game_backup_dir.is_dir() {
        bail!(
            "Given path ({}) is not a directory",
            args.game_backup_dir.display()
        )
    }

    let game_paths: Vec<_> = args
        .game_backup_dir
        .read_dir()?
        .filter_map(|entry_result| {
            let entry = entry_result.ok()?;
            let path = entry.path();
            let ext = path.extension()?;
            (ext == "nsp").then_some(path.into_os_string().into_string().unwrap() + "\n")
        })
        .collect();
    if game_paths.is_empty() {
        bail!(
            "No game backup files found in given directory ({})",
            args.game_backup_dir.display()
        )
    }
    let all_paths_string_length = game_paths.iter().fold(0, |acc, path| acc + path.len());

    let device_info = list_devices()
        .wait()?
        .find(|dev| dev.vendor_id() == 0x57e && dev.product_id() == 0x3000)
        .wrap_err("Unable to discover Nintendo Switch through USB.")
        .suggestion(
            "Ensure the Nintendo Switch is awake and connected via cable to this computer.",
        )?;

    info!(
        "Nintendo Switch discovered at bus {} and address {}",
        device_info.bus_id(),
        device_info.device_address()
    );

    let device = device_info.open().wait()?;
    let interface = device.claim_interface(0).wait()?;
    let mut ep_out = interface.endpoint::<Bulk, Out>(0x01)?;
    ep_out.clear_halt().wait()?;
    let mut ep_in = interface.endpoint::<Bulk, In>(0x81)?;
    ep_in.clear_halt().wait()?;

    debug!("sending game backup list");
    write_usb(&mut ep_out, "TUL0")?;
    write_usb(&mut ep_out, &all_paths_string_length.to_le_bytes()[..4])?;
    write_usb(&mut ep_out, [0u8; 8])?;
    for path in &game_paths {
        write_usb(&mut ep_out, path.as_str())?;
    }

    let mut pb = ProgressBar::no_length().with_style(
        ProgressStyle::with_template("ETA: {eta} ({binary_bytes_per_sec}) {wide_bar} {binary_bytes} of {binary_total_bytes} sent").unwrap(),
    );

    loop {
        debug!("waiting for header...");
        let command_header = ep_in
            .transfer_blocking(Buffer::new(512), Duration::MAX)
            .into_result()?;
        debug!("got header: {:#?}", &command_header);

        if &command_header[..4] != b"TUC0" {
            error!("invalid command header magic. continuing to next iteration...");
            continue;
        }
        debug!("correct command header magic");

        let command_type: [u8; 1] = command_header[4..5].try_into().unwrap();
        let command_id: [u8; 4] = command_header[8..12].try_into().unwrap();

        debug!(
            "Command type: {:?}, Command id: {:?}",
            &command_type, &command_id
        );

        match command_id {
            tinfoil_command_ids::EXIT => {
                debug!("got exit command, exiting...");
                pb.finish();
                break;
            }
            tinfoil_command_ids::FILE_RANGE => {
                debug!("got file range command");
                file_range_command(&mut ep_in, &mut ep_out, &mut pb, &game_paths)?
            }
            _ => bail!("invalid command ID encountered!"),
        }
    }

    Ok(())
}
