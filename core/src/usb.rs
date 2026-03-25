use color_eyre::eyre::{Context, ContextCompat};
use color_eyre::eyre::{bail, eyre};
use log::{debug, error, info};
use nusb::{
    Endpoint, MaybeFuture, list_devices,
    transfer::{Buffer, Bulk, In, Out, TransferError},
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::Path,
    time::Duration,
};
use thiserror::Error;

const USB_TIMEOUT: Duration = Duration::from_millis(500);

mod tinfoil {
    use std::path::PathBuf;

    use super::*;

    pub mod command_direction {
        pub const RESPONSE: [u8; 4] = 0u32.to_le_bytes();
    }
    pub mod command {
        pub const EXIT: [u8; 4] = 0u32.to_le_bytes();
        pub const FILE_RANGE: [u8; 4] = 1u32.to_le_bytes();
    }

    pub fn send_response_header(
        ep_out: &mut Endpoint<Bulk, Out>,
        range_size: usize,
    ) -> color_eyre::Result<()> {
        write_usb(ep_out, b"TUC0")?;
        write_usb(ep_out, tinfoil::command_direction::RESPONSE)?;
        write_usb(ep_out, tinfoil::command::FILE_RANGE)?;
        write_usb(ep_out, range_size.to_le_bytes())?;
        write_usb(ep_out, [0u8; 0xC])?; // padding?
        Ok(())
    }

    pub fn file_range_command(
        ep_in: &mut Endpoint<Bulk, In>,
        ep_out: &mut Endpoint<Bulk, Out>,
        game_paths: &[PathBuf],
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
            let file_size = metadata.len();
            progress_len_tx.send(file_size)?;
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

    pub fn do_workloop(
        ep_in: &mut Endpoint<Bulk, In>,
        ep_out: &mut Endpoint<Bulk, Out>,
        cancel: Option<Arc<AtomicBool>>,
        game_paths: &[PathBuf],
        progress_len_tx: mpsc::Sender<u64>,
        progress_tx: mpsc::Sender<u64>,
    ) -> color_eyre::Result<()> {
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
                tinfoil::command::EXIT => {
                    debug!("got exit command, exiting...");
                    break;
                }
                tinfoil::command::FILE_RANGE => {
                    debug!("got file range command");
                    tinfoil::file_range_command(
                        ep_in,
                        ep_out,
                        game_paths,
                        &progress_len_tx,
                        &progress_tx,
                    )?
                }
                _ => bail!("invalid tinfoil command encountered!"),
            }
        }
        Ok(())
    }
}

mod sphaira {
    use std::path::PathBuf;

    use color_eyre::eyre::Context;

    use super::*;

    const EXPECTED_MAGIC: u32 = u32::from_be_bytes(*b"SPH0");
    const PACKET_SIZE: usize = 24;

    pub const RESULT_OK: u32 = 0;
    pub const RESULT_ERROR: u32 = 1;

    mod command {
        pub const QUIT: u32 = 0;
        pub const OPEN: u32 = 1;
    }

    const FLAG_NONE: u32 = 0;

    #[allow(unused)]
    const FLAG_STREAM: u32 = 1;

    struct SendHeader {
        arg2: u32,
        arg3: u32,
        arg4: u32,
    }

    impl SendHeader {
        fn get_offset(&self) -> u64 {
            ((self.arg2 as u64) << 32) | (self.arg3 as u64)
        }
        fn get_size(&self) -> usize {
            self.arg4 as usize
        }
    }

    /// discard the send header, just validate
    pub fn validate_send_header(ep_in: &mut Endpoint<Bulk, In>) -> color_eyre::Result<()> {
        let _ = get_send_header(ep_in)?;
        Ok(())
    }

    #[derive(Debug, Error)]
    enum SendHeaderError {
        #[error("Invalid magic in Sphaira send header")]
        InvalidMagic,
        #[error("Invalid CRC32C in Sphaira send header")]
        InvalidCrc32c,
        #[error(transparent)]
        TransferError(#[from] TransferError),
    }

    /// read send header and check magic
    fn get_send_header(ep_in: &mut Endpoint<Bulk, In>) -> Result<SendHeader, SendHeaderError> {
        let response = read_usb(ep_in)?;

        let mut args = response.chunks(4);

        let [magic, arg2, arg3, arg4, _arg5, crc32c] =
            std::array::from_fn(|_| u32::from_le_bytes(args.next().unwrap().try_into().unwrap()));

        if magic != EXPECTED_MAGIC {
            error!(
                "Invalid magic in Sphaira send header. Expected: {EXPECTED_MAGIC:?}, Given: {magic:?}"
            );
            return Err(SendHeaderError::InvalidMagic);
        }

        let computed_crc32c = crc32c::crc32c(&response[0..4 * 5]);
        if computed_crc32c != crc32c {
            error!(
                "Invalid CRC32C in Sphaira send header. Computed: {computed_crc32c:?}, Given: {crc32c:?}"
            );
            return Err(SendHeaderError::InvalidCrc32c);
        }

        Ok(SendHeader { arg2, arg3, arg4 })
    }

    pub fn send_result(
        ep_out: &mut Endpoint<Bulk, Out>,
        result: u32,
        arg3: impl Into<Option<u32>>,
        arg4: impl Into<Option<u32>>,
    ) -> color_eyre::Result<()> {
        let arg3 = arg3.into();
        let arg4 = arg4.into();

        let mut packet = [0u8; PACKET_SIZE];
        packet[..4].copy_from_slice(&EXPECTED_MAGIC.to_le_bytes());
        packet[4..8].copy_from_slice(&result.to_le_bytes());
        packet[8..12].copy_from_slice(&arg3.unwrap_or(0).to_le_bytes());
        packet[12..16].copy_from_slice(&arg4.unwrap_or(0).to_le_bytes());

        let crc32c = crc32c::crc32c(&packet[0..4 * 5]);
        packet[4 * 5..4 * 6].copy_from_slice(&crc32c.to_le_bytes());

        write_usb(ep_out, packet)
    }

    fn do_file_transfer_loop(
        ep_in: &mut Endpoint<Bulk, In>,
        ep_out: &mut Endpoint<Bulk, Out>,
        cancel: Option<&AtomicBool>,
        mut file_reader: impl Read + Seek,
        progress_tx: &mpsc::Sender<u64>,
    ) -> color_eyre::Result<()> {
        loop {
            if cancel.is_some_and(|c| c.load(Ordering::Relaxed)) {
                info!("cancellation requested, exiting...");
                send_result(ep_out, RESULT_ERROR, None, None)?;
                return Ok(());
            }

            let send_header = match get_send_header(ep_in) {
                Ok(header) => header,
                Err(SendHeaderError::TransferError(
                    TransferError::Cancelled | TransferError::Disconnected,
                )) => {
                    info!("cancellation requested, hopefully means we are done with this file!");
                    continue;
                }
                Err(e) => {
                    send_result(ep_out, RESULT_ERROR, None, None)?;
                    return Err(e.into());
                }
            };
            let offset = send_header.get_offset();
            let size = send_header.get_size();

            let _ = progress_tx.send(offset);

            if (offset == 0) && (size == 0) {
                debug!("file transfer complete!");
                send_result(ep_out, RESULT_OK, None, None)?;
                break;
            }

            file_reader.seek(SeekFrom::Start(offset))?;
            let mut buf = vec![0u8; size];

            let bytes_read = match file_reader.read(&mut buf) {
                Ok(bytes_read) => bytes_read,
                Err(e) => {
                    error!("Failed to read file data for Sphaira client: {e:?}");
                    send_result(ep_out, RESULT_ERROR, None, None)?;
                    continue;
                }
            };

            send_result(ep_out, RESULT_OK, bytes_read as u32, crc32c::crc32c(&buf))?;
            write_usb(ep_out, buf)?;
        }

        Ok(())
    }

    fn transfer_single_file(
        ep_in: &mut Endpoint<Bulk, In>,
        ep_out: &mut Endpoint<Bulk, Out>,
        cancel: Option<&AtomicBool>,
        game_path: impl AsRef<Path>,
        progress_len_tx: &mpsc::Sender<u64>,
        progress_tx: &mpsc::Sender<u64>,
    ) -> color_eyre::Result<()> {
        let game_path = game_path.as_ref();
        info!("transferring file {}...", game_path.display());

        // TODO: this is hardcoded for now, should be different if streaming .rar files
        let flags = FLAG_NONE;

        let file_size = game_path
            .metadata()
            .wrap_err_with(|| {
                format!(
                    "Failed to read filesize of game path {} requested by Sphaira client",
                    game_path.display()
                )
            })?
            .len();

        let _ = progress_len_tx.send(file_size);

        // WARNING: probably is wrong, has bugs..
        let size_lsb = file_size as u32;
        let size_msb = ((file_size >> 32) as u16) as u32 | (flags << 16);
        send_result(ep_out, RESULT_OK, size_msb, size_lsb)?;

        let file = File::open(game_path)?;
        let file_reader = BufReader::new(file);
        do_file_transfer_loop(ep_in, ep_out, cancel, file_reader, progress_tx)?;

        Ok(())
    }

    pub fn do_workloop(
        ep_in: &mut Endpoint<Bulk, In>,
        ep_out: &mut Endpoint<Bulk, Out>,
        cancel: Option<Arc<AtomicBool>>,
        game_paths: &[PathBuf],
        progress_len_tx: mpsc::Sender<u64>,
        progress_tx: mpsc::Sender<u64>,
    ) -> color_eyre::Result<()> {
        loop {
            let SendHeader {
                arg2: cmd,
                arg3: game_path_index,
                ..
            } = match get_send_header(ep_in) {
                Ok(header) => header,
                Err(SendHeaderError::TransferError(
                    TransferError::Cancelled | TransferError::Disconnected,
                )) => {
                    info!("cancellation requested, exiting...");
                    return Ok(());
                }
                Err(e) => {
                    send_result(ep_out, RESULT_ERROR, None, None)?;
                    return Err(e.into());
                }
            };

            match cmd {
                command::QUIT => send_result(ep_out, RESULT_OK, None, None)?,
                command::OPEN => {
                    let game_path = game_paths.get(game_path_index as usize).wrap_err_with(||
                        format!(
                            "Sphaira client requested game index {game_path_index}, but only {} games were sent",
                            game_paths.len()
                        )
                    )?;
                    transfer_single_file(
                        ep_in,
                        ep_out,
                        // FIXME: fucked up!!! avoid Arc nonsense within this file!
                        cancel.as_ref().map(|c| c.as_ref()),
                        game_path,
                        // FIXME: fucked up!!! refernence of Sender!?
                        &progress_len_tx,
                        // FIXME: fucked up!!! refernence of Sender!?
                        &progress_tx,
                    )?;
                }
                _ => {
                    send_result(ep_out, RESULT_ERROR, None, None)?;
                    bail!("Invalid command from Sphaira client: {cmd}");
                }
            }
        }
    }
}

pub fn perform_usb_install(
    game_paths: &[PathBuf],
    progress_len_tx: mpsc::Sender<u64>,
    progress_tx: mpsc::Sender<u64>,
    for_sphaira: bool,
    cancel: impl Into<Option<Arc<AtomicBool>>>,
) -> color_eyre::Result<()> {
    let cancel = cancel.into();
    let paths_with_newlines_string_length: u32 = game_paths
        .iter()
        .map(|path| path.to_str().unwrap().len() as u32 + 1)
        .sum();

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
                    "Permission denied opening USB connection to Nintendo Switch\nEnsure you have read-write permissions for USB device at bus {} and address {}",
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

    if for_sphaira {
        sphaira::validate_send_header(&mut ep_in).wrap_err(
            [
                "Failed to perform initial handshake with Sphaira.",
                "Ensure Sphaira is open on the Nintendo Switch, and in the menu 'USB Install'.",
                "If you are trying to transfer to Awoo Installer or CyberFoil, use the regular tinfoil USB install option instead.",
            ]
            .join("\n"),
        )?;
        sphaira::send_result(
            &mut ep_out,
            sphaira::RESULT_OK,
            Some(paths_with_newlines_string_length),
            None,
        )?;
        write_usb(
            &mut ep_out,
            game_paths.iter().fold(String::new(), |acc, path| {
                acc + path.to_str().unwrap() + "\n"
            }),
        )?;
    } else {
        write_usb(&mut ep_out, "TUL0")?;
        write_usb(&mut ep_out, paths_with_newlines_string_length.to_le_bytes())?;
        write_usb(&mut ep_out, [0u8; 8])?;
        for path in game_paths {
            write_usb(&mut ep_out, [path.to_str().unwrap(), "\n"].concat())?;
        }
    }

    eprintln!("Sent list of games to Nintendo Switch.");

    if for_sphaira {
        eprintln!("Starting Sphaira USB install.");
        sphaira::do_workloop(
            &mut ep_in,
            &mut ep_out,
            cancel,
            &game_paths,
            progress_len_tx,
            progress_tx,
        )
        .inspect_err(|_| {
            let _ = sphaira::send_result(&mut ep_out, sphaira::RESULT_ERROR, None, None);
        })?;
    } else {
        eprintln!("Starting tinfoil USB install.");
        tinfoil::do_workloop(
            &mut ep_in,
            &mut ep_out,
            cancel,
            &game_paths,
            progress_len_tx,
            progress_tx,
        )?;
    }

    let num_games_installed = game_paths.len();
    eprintln!(
        "Installed {} game{} over USB successfully!",
        num_games_installed,
        if num_games_installed == 1 { "" } else { "s" }
    );

    Ok(())
}

fn read_usb(ep_in: &mut Endpoint<Bulk, In>) -> Result<Buffer, TransferError> {
    // TODO: avoid creating buffer everytime?
    // TODO: figure out if 512 is universal buffer size or just my machine?
    let buf = Buffer::new(512);
    ep_in.transfer_blocking(buf, USB_TIMEOUT).into_result()
}

fn write_usb(
    ep_out: &mut Endpoint<Bulk, Out>,
    message: impl Into<Vec<u8>>,
) -> color_eyre::Result<()> {
    let buf = message.into();
    ep_out
        .transfer_blocking(buf.into(), USB_TIMEOUT)
        .status
        .map_err(|e| match e {
            TransferError::Cancelled => {
                eyre!(
                    [
                        "Nintendo Switch was discovered, but it is not accepting transfers.",
                        "Ensure you are in the USB install menu."
                    ]
                    .join("\n")
                )
            }
            TransferError::Disconnected => eyre!("USB has disconnected"),
            TransferError::Fault | TransferError::Stall | TransferError::InvalidArgument => {
                eyre!("Malformed data during transfer. {:?}", e)
            }
            TransferError::Unknown(i) => eyre!("Unknown error {}", i),
        })
}
