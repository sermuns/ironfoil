use color_eyre::eyre::{Context, ContextCompat, bail};
use log::{debug, error, info};
use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
use thiserror::Error;

use crate::{InstallProgressEvent, InstallProgressSender};

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

pub struct SendHeader {
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

#[derive(Debug, Error)]
pub enum SendHeaderError {
    #[error("Invalid magic in Sphaira send header")]
    InvalidMagic,
    #[error("Invalid CRC32C in Sphaira send header")]
    InvalidCrc32c,
    #[error("Nintendo Switch disconnected. Sphaira does this on successful install completion.")]
    Disconnected,
    #[error(transparent)]
    OtherIo(#[from] std::io::Error),
}

/// read send header and check magic
pub fn get_send_header(usb_reader: &mut impl Read) -> Result<SendHeader, SendHeaderError> {
    let mut response = [0u8; PACKET_SIZE];
    usb_reader.read_exact(&mut response).map_err(|e| {
        if e.kind() == std::io::ErrorKind::ConnectionAborted {
            SendHeaderError::Disconnected
        } else {
            SendHeaderError::OtherIo(e)
        }
    })?;

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
    usb_writer: &mut impl Write,
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

    usb_writer.write_all(&packet)?;
    usb_writer.flush()?;
    Ok(())
}

fn do_single_file_transfer_loop(
    usb_reader: &mut impl Read,
    usb_writer: &mut impl Write,
    cancel: Option<&AtomicBool>,
    mut file_reader: impl Read + Seek,
    progress_tx: &InstallProgressSender,
    all_files_offset_bytes: &mut u64,
) -> color_eyre::Result<()> {
    loop {
        if cancel.is_some_and(|c| c.load(Ordering::Relaxed)) {
            info!("cancellation requested, exiting...");
            send_result(usb_writer, RESULT_ERROR, None, None)?;
            return Ok(());
        }

        let send_header = match get_send_header(usb_reader) {
            Ok(header) => header,
            Err(SendHeaderError::Disconnected) => {
                info!("switch disconnected, hopefully means we are done with this file!");
                continue;
            }
            Err(e) => return Err(e.into()),
        };
        let offset = send_header.get_offset();
        let size = send_header.get_size();

        let _ = progress_tx.send(InstallProgressEvent::CurrentFileOffsetBytes(offset));

        if (offset == 0) && (size == 0) {
            debug!("file transfer complete!");
            send_result(usb_writer, RESULT_OK, None, None)?;
            break;
        }

        file_reader.seek(SeekFrom::Start(offset))?;
        let mut buf = vec![0u8; size];

        let bytes_read = file_reader.read(&mut buf)?;

        send_result(
            usb_writer,
            RESULT_OK,
            bytes_read as u32,
            crc32c::crc32c(&buf),
        )?;
        usb_writer.write_all(&buf)?;
        usb_writer.flush()?;

        *all_files_offset_bytes += size as u64;
        let _ = progress_tx.send(InstallProgressEvent::AllFilesOffsetBytes(
            *all_files_offset_bytes,
        ));
    }

    Ok(())
}

fn transfer_single_file(
    usb_reader: &mut impl Read,
    usb_writer: &mut impl Write,
    cancel: Option<&AtomicBool>,
    game_path: impl AsRef<Path>,
    progress_tx: &InstallProgressSender,
    all_files_offset_bytes: &mut u64,
) -> color_eyre::Result<()> {
    let game_path = game_path.as_ref();
    info!("transferring file {}...", game_path.display());
    let _ = progress_tx.send(InstallProgressEvent::CurrentFileName(
        game_path.file_name().unwrap().to_string_lossy().to_string(),
    ));

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

    let _ = progress_tx.send(InstallProgressEvent::CurrentFileLengthBytes(file_size));

    let size_low_bits = file_size as u32;
    let size_high_bits = u32::from((file_size >> 32) as u16) | (flags << 16);
    send_result(usb_writer, RESULT_OK, size_high_bits, size_low_bits)?;

    let file = File::open(game_path)?;
    let file_reader = BufReader::new(file);
    do_single_file_transfer_loop(
        usb_reader,
        usb_writer,
        cancel,
        file_reader,
        progress_tx,
        all_files_offset_bytes,
    )?;

    Ok(())
}

pub fn do_workloop(
    usb_reader: &mut impl Read,
    usb_writer: &mut impl Write,
    cancel: Option<&AtomicBool>,
    game_paths: &[PathBuf],
    progress_tx: &InstallProgressSender,
) -> color_eyre::Result<()> {
    let mut all_files_offset_bytes = 0;

    loop {
        let SendHeader {
            arg2: cmd,
            arg3: game_path_index,
            ..
        } = match get_send_header(usb_reader) {
            Ok(header) => header,
            Err(SendHeaderError::Disconnected) => {
                info!("cancellation requested, exiting...");
                return Ok(());
            }
            Err(e) => {
                error!("Failed to read command header from Sphaira client: {e:?}");
                return Err(e.into());
            }
        };

        match cmd {
            command::QUIT => send_result(usb_writer, RESULT_OK, None, None)?,
            command::OPEN => {
                let game_path = game_paths.get(game_path_index as usize).wrap_err_with(||
                        format!(
                            "Sphaira client requested game index {game_path_index}, but only {} games were sent",
                            game_paths.len()
                        )
                    )?;
                transfer_single_file(
                    usb_reader,
                    usb_writer,
                    cancel,
                    game_path,
                    progress_tx,
                    &mut all_files_offset_bytes,
                )?;
            }
            _ => bail!("Invalid command from Sphaira client: {cmd}"),
        }
    }
}

pub fn initiate_transfer(
    usb_reader: &mut impl Read,
    usb_writer: &mut impl Write,
    paths_with_newlines_string_length: u32,
    game_paths: &[PathBuf],
) -> color_eyre::Result<()> {
    // just validate
    let _ = get_send_header(usb_reader).wrap_err(
                [
                    "Failed to perform initial handshake with Sphaira.",
                    "Ensure Sphaira is open on the Nintendo Switch, and in the menu 'USB Install'.",
                    "If you are trying to transfer to Awoo Installer or CyberFoil, you need to change to that USB install option.",
                ]
                .join("\n"),
            )?;
    send_result(
        usb_writer,
        RESULT_OK,
        Some(paths_with_newlines_string_length),
        None,
    )?;
    for path in game_paths {
        writeln!(usb_writer, "{}", path.to_str().unwrap())?;
    }
    usb_writer.flush()?;

    Ok(())
}
