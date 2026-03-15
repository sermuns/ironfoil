use color_eyre::eyre::ContextCompat;
use color_eyre::eyre::{bail, eyre};
use log::{debug, error, info};
use nusb::{
    Endpoint, MaybeFuture, list_devices,
    transfer::{Buffer, Bulk, In, Out, TransferError},
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::sleep;
use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::Path,
    time::Duration,
};

use crate::paths::read_game_paths;

const USB_TIMEOUT: Duration = Duration::from_millis(500);

mod tinfoil_command_types {
    pub const RESPONSE: [u8; 4] = 0u32.to_le_bytes();
}

mod tinfoil_command_ids {
    pub const EXIT: [u8; 4] = 0u32.to_le_bytes();
    pub const FILE_RANGE: [u8; 4] = 1u32.to_le_bytes();
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
    game_paths: &[String],
    progress_len_tx: &mpsc::Sender<u64>,
    progress_tx: &mpsc::Sender<u64>,
) -> color_eyre::Result<()> {
    let file_range_header = read_usb(ep_in)?;

    let range_size = usize::from_le_bytes(file_range_header[..8].try_into().unwrap());
    let range_offset = u64::from_le_bytes(file_range_header[8..16].try_into().unwrap());
    let game_path_len = usize::from_le_bytes(file_range_header[16..24].try_into().unwrap());

    let game_name_buf = read_usb(ep_in)?;
    let game_path = str::from_utf8(&game_name_buf)?;

    if !game_paths.iter().any(|path| game_path == path) {
        bail!(
            "Nintendo Switch tried to request game backup ({}) not present on host",
            game_path
        );
    };

    info!("sending {}", &game_path);

    info!(
        "Range size: {}, Range offset: {}, Name len: {}, Name: {}",
        range_size, range_offset, game_path_len, game_path,
    );

    send_response_header(ep_out, range_size)?;

    let file = File::open(game_path)?;

    if let Ok(metadata) = file.metadata() {
        progress_len_tx.send(metadata.len())?;
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
        progress_tx.send(current_offset as u64)?;
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

pub fn perform_tinfoil_usb_install(
    game_backup_path: &Path,
    recurse: bool,
    progress_len_tx: mpsc::Sender<u64>,
    progress_tx: mpsc::Sender<u64>,
    cancel: impl Into<Option<Arc<AtomicBool>>>,
) -> color_eyre::Result<()> {
    let cancel = cancel.into();
    let game_paths = read_game_paths(game_backup_path, recurse, cancel.as_deref())?;
    let paths_with_newlines_string_length: usize =
        game_paths.iter().map(|path| path.len() + 1).sum();

    let device_info = list_devices()
        .wait()?
        .find(|dev| dev.vendor_id() == 0x57e && dev.product_id() == 0x3000)
        .wrap_err(
            [
                "Unable to discover Nintendo Switch through USB.",
                "Ensure the Nintendo Switch is awake and connected via cable to this computer.",
            ]
            .join("\n"),
        )?;

    info!(
        "Nintendo Switch discovered at bus {} and address {}",
        device_info.bus_id(),
        device_info.device_address()
    );

    let device = device_info
        .open()
        .wait()
        .map_err(|e| match (e.kind(), e.os_error()) {
            (nusb::ErrorKind::PermissionDenied, _) | (nusb::ErrorKind::Other, Some(13)) => {
                eyre!(
                    "Permission denied opening USB connection to Nintendo Switch\nEnsure you have read-write permissions for bus {} at address {}",
                    device_info.bus_id(),
                    device_info.device_address(),
                )
            }
            _ => eyre!("Failed to open USB connection to Nintendo Switch: {:?}", e),
        })?;
    let interface = device.claim_interface(0).wait()?;
    let mut ep_out = interface.endpoint::<Bulk, Out>(0x01)?;
    ep_out.clear_halt().wait()?;
    let mut ep_in = interface.endpoint::<Bulk, In>(0x81)?;
    ep_in.clear_halt().wait()?;

    write_usb(&mut ep_out, "TUL0")?;
    write_usb(
        &mut ep_out,
        &paths_with_newlines_string_length.to_le_bytes()[..4],
    )?; // FIXME: ugly slicing
    write_usb(&mut ep_out, [0u8; 8])?;
    for path in &game_paths {
        write_usb(&mut ep_out, [path.as_str(), "\n"].concat())?;
    }

    eprintln!("Successfully sent list of games to Nintendo Switch, waiting for commands...");

    loop {
        if cancel.as_ref().is_some_and(|c| c.load(Ordering::Relaxed)) {
            info!("cancellation requested, exiting...");
            return Ok(());
        }
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
                break;
            }
            tinfoil_command_ids::FILE_RANGE => {
                debug!("got file range command");
                file_range_command(
                    &mut ep_in,
                    &mut ep_out,
                    &game_paths,
                    &progress_len_tx,
                    &progress_tx,
                )?
            }
            _ => bail!("invalid command ID encountered!"),
        }
    }

    Ok(())
}

fn write_usb(
    ep_out: &mut Endpoint<Bulk, Out>,
    message: impl Into<Vec<u8>>,
) -> color_eyre::Result<()> {
    let buf_vec = message.into();
    for _ in 0..50 {
        match ep_out
            // FIXME: shitty clone per retry iteration
            .transfer_blocking(buf_vec.clone().into(), USB_TIMEOUT)
            .status
        {
            Ok(_) => return Ok(()),
            Err(e) => match e {
                TransferError::Cancelled => {
                    const RETRY_DURATION: Duration = Duration::from_millis(500);
                    eprintln!(
                        "Transfer was cancelled, retrying in {:.1} seconds...",
                        RETRY_DURATION.as_secs_f32()
                    );
                    if let Err(e) = ep_out.clear_halt().wait() {
                        error!("failed to clear halt on endpoint after cancelled transfer: {e:?}",);
                    }
                    sleep(RETRY_DURATION);
                }
                TransferError::Disconnected => bail!("USB has disconnected"),
                TransferError::Fault | TransferError::Stall | TransferError::InvalidArgument => {
                    bail!("Malformed data during transfer. {:?}", e)
                }
                TransferError::Unknown(i) => bail!("Unknown error {}", i),
            },
        }
    }
    bail!(
        [
            "Nintendo Switch was discovered, but it is not accepting transfers.",
            "Ensure Awoo Installer is open, and in the menu 'Install Over USB'."
        ]
        .join("\n")
    )
}
