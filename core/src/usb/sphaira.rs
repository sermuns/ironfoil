use color_eyre::eyre::{Context, ContextCompat, bail};
use log::{debug, error, info};
use nusb::{
    Endpoint,
    transfer::{Bulk, In, Out, TransferError},
};
use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};
use thiserror::Error;

use crate::usb::{read_usb, write_usb};

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
        (u64::from(self.arg2) << 32) | u64::from(self.arg3)
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

    let mut chunks = response.chunks(4);

    let [magic, arg2, arg3, arg4, _, crc32c] =
        std::array::from_fn(|_| u32::from_le_bytes(chunks.next().unwrap().try_into().unwrap()));

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

    let size_lsb = file_size as u32;
    let size_msb = u32::from((file_size >> 32) as u16) | (flags << 16);
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
                    cancel.as_ref().map(AsRef::as_ref),
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
