// TODO: handle transfer cancelled gracefully

use clap::Parser;
use color_eyre::{
    Section,
    config::Theme,
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
            TransferError::Cancelled => eyre!("Nintendo Switch is not accepting transfers.\n Is Awoo Installer open in the menu 'Install Over USB'?"),
            TransferError::Fault | TransferError::Stall => eyre!("USB connection error"),
            TransferError::Disconnected => eyre!("USB has disconnected"),
            TransferError::InvalidArgument => eyre!("Malformed data during transfer... oops!"),
            TransferError::Unknown(i) => eyre!("Unknown error {}", i),
        })
}

fn read_usb(ep_in: &mut Endpoint<Bulk, In>) -> Result<Buffer, TransferError> {
    // TODO: avoid creating buffer everytime?
    // TODO: figure out if 512 is universal buffer size or just my machine?
    let buf = Buffer::new(512);
    ep_in.transfer_blocking(buf, USB_TIMEOUT).into_result()
}

#[derive(Parser)]
struct Args {
    nsp_dir: PathBuf,
}

fn main() -> color_eyre::Result<()> {
    env_logger::builder().format_source_path(true).init();
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .display_location_section(false)
        .install()?;

    let args = Args::parse();

    if !args.nsp_dir.exists() {
        bail!("Given path ({}) does not exist", args.nsp_dir.display())
    }
    if !args.nsp_dir.is_dir() {
        bail!("Given path ({}) is not a directory", args.nsp_dir.display())
    }

    let nsp_paths: Vec<_> = args
        .nsp_dir
        .read_dir()?
        .filter_map(|entry_result| {
            let entry = entry_result.ok()?;
            let path = entry.path();
            (path.extension()? == "nsp")
                .then_some(path.into_os_string().into_string().unwrap() + "\n")
        })
        .collect();
    if nsp_paths.is_empty() {
        bail!(
            "No NSPs found in given directory ({})",
            args.nsp_dir.display()
        )
    }
    let all_paths_string_length = nsp_paths.iter().fold(0, |acc, path| acc + path.len());

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

    debug!("sending nsp list");
    write_usb(&mut ep_out, "TUL0")?;
    write_usb(&mut ep_out, &all_paths_string_length.to_le_bytes()[..4])?;
    write_usb(&mut ep_out, [0u8; 8])?;
    for nsp_path in &nsp_paths {
        write_usb(&mut ep_out, nsp_path.as_str())?;
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
        // let data_size = u64::from_le_bytes(command_header[12..20].try_into().unwrap());

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
                file_range_command(&mut ep_in, &mut ep_out, &mut pb, &nsp_paths)?
            }
            _ => bail!("invalid command ID encountered!"),
        }
    }

    Ok(())
}

fn file_range_command(
    ep_in: &mut Endpoint<Bulk, In>,
    ep_out: &mut Endpoint<Bulk, Out>,
    pb: &mut ProgressBar,
    nsps: &[String],
) -> color_eyre::Result<()> {
    let file_range_header = read_usb(ep_in)?;

    let range_size = usize::from_le_bytes(file_range_header[..8].try_into().unwrap());
    let range_offset = u64::from_le_bytes(file_range_header[8..16].try_into().unwrap());
    let nsp_name_len = usize::from_le_bytes(file_range_header[16..24].try_into().unwrap());

    let nsp_name_buf = read_usb(ep_in)?;
    let nsp_path = str::from_utf8(&nsp_name_buf)?;

    if !nsps
        .iter()
        .any(|path| path.len() == nsp_path.len() + 1 && *nsp_path == path[..nsp_path.len()])
    {
        warn!("{:#?}", nsps);
        warn!("requested: {:#?}", nsp_path);
        bail!("Nintendo Switch tried to request NSP not present on host");
    };

    info!("sending {}", &nsp_path);

    info!(
        "Range size: {}, Range offset: {}, Name len: {}, Name: {}",
        range_size, range_offset, nsp_name_len, nsp_path,
    );

    send_response_header(ep_out, range_size)?;

    let file = File::open(nsp_path)?;

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

    // TODO: a single u32?
    write_usb(ep_out, tinfoil_command_types::RESPONSE)?;
    write_usb(ep_out, [0u8; 3])?;

    write_usb(ep_out, tinfoil_command_ids::FILE_RANGE)?;

    // TODO: also simplify this padding?
    write_usb(ep_out, range_size.to_le_bytes())?;
    write_usb(ep_out, [0u8; 0xC])?;

    Ok(())
}

mod tinfoil_command_types {
    pub const RESPONSE: [u8; 1] = [0u8];
}

mod tinfoil_command_ids {
    pub const EXIT: [u8; 4] = 0u32.to_le_bytes();
    pub const FILE_RANGE: [u8; 4] = 1u32.to_le_bytes();
}
