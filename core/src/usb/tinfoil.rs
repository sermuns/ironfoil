use color_eyre::eyre::bail;
use log::{debug, error, info};
use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{InstallProgressEvent, InstallProgressSender};

pub mod command_direction {
    pub const RESPONSE: [u8; 4] = 0u32.to_le_bytes();
}
pub mod command {
    pub const EXIT: [u8; 4] = 0u32.to_le_bytes();
    pub const FILE_RANGE: [u8; 4] = 1u32.to_le_bytes();
}

pub fn send_response_header(
    usb_writer: &mut impl Write,
    range_size: usize,
) -> color_eyre::Result<()> {
    usb_writer.write_all(b"TUC0")?;
    usb_writer.flush()?;

    usb_writer.write_all(&command_direction::RESPONSE)?;
    usb_writer.flush()?;

    usb_writer.write_all(&command::FILE_RANGE)?;
    usb_writer.flush()?;

    usb_writer.write_all(&range_size.to_le_bytes())?;
    usb_writer.flush()?;

    usb_writer.write_all(&[0u8; 0xC])?;
    usb_writer.flush()?;

    Ok(())
}

pub struct CachedGameFile {
    path: PathBuf,
    reader: BufReader<File>,
    // size: u64,
}

pub fn file_range_command(
    usb_reader: &mut impl Read,
    usb_writer: &mut impl Write,
    buf: &mut [u8],
    cached_game_file: &mut Option<CachedGameFile>,
    game_paths: &[PathBuf],
    progress_tx: &InstallProgressSender,
    all_files_offset_bytes: &mut u64,
) -> color_eyre::Result<()> {
    let mut file_range_header = [0u8; 32];
    usb_reader.read_exact(&mut file_range_header)?;

    debug!("got file range header: {:#?}", &file_range_header);

    let range_size = usize::from_le_bytes(file_range_header[..8].try_into().unwrap());
    let range_offset = u64::from_le_bytes(file_range_header[8..16].try_into().unwrap());
    let game_path_utf8_len = usize::from_le_bytes(file_range_header[16..24].try_into().unwrap());
    info!("got game path utf8 len: {}", game_path_utf8_len);

    let num_bytes_in_game_path = usb_reader.read(buf)?;
    let game_path_str = str::from_utf8(&buf[..num_bytes_in_game_path])?;

    info!(
        "requested file range with path: {}, offset: {}, size: {}",
        game_path_str, range_offset, range_size
    );

    let game_path = Path::new(game_path_str);

    if !game_paths.iter().any(|path| game_path == path) {
        bail!(
            "Nintendo Switch tried to request game backup ({}) not present on host",
            game_path.display()
        );
    }

    let game_name = game_path.file_name().unwrap().to_str().unwrap();

    info!(
        "Range size: {}, Range offset: {}, Name len: {}, Name: {}",
        range_size, range_offset, game_path_utf8_len, game_name,
    );

    send_response_header(usb_writer, range_size)?;

    let file = if let Some(cached) = cached_game_file
        && cached.path == game_path
    {
        info!("reusing cached file");
        cached
    } else {
        info!("new file requested, caching it and dropping old cached file");
        let _ = progress_tx.send(InstallProgressEvent::CurrentFileName(game_name.to_string()));

        let file = File::open(game_path)?;

        let size = file.metadata()?.len();
        let _ = progress_tx.send(InstallProgressEvent::CurrentFileLengthBytes(size));

        let new_cached_game_file = CachedGameFile {
            path: PathBuf::from(game_path),
            reader: BufReader::new(file),
            // size,
        };

        cached_game_file.insert(new_cached_game_file)
    };

    file.reader.seek(SeekFrom::Start(range_offset))?;

    let mut remaining_bytes_in_file = range_size;
    let mut current_file_offset_bytes = range_offset;

    let mut send_buf = [0u8; 2usize.pow(14)];

    while remaining_bytes_in_file > 0 {
        let chunk_size = remaining_bytes_in_file.min(send_buf.len());
        let chunk = &mut send_buf[..chunk_size];

        file.reader.read_exact(chunk)?;
        usb_writer.write_all(chunk)?;

        remaining_bytes_in_file -= chunk_size;

        current_file_offset_bytes += chunk_size as u64;
        let _ = progress_tx.send(InstallProgressEvent::CurrentFileOffsetBytes(
            current_file_offset_bytes,
        ));

        *all_files_offset_bytes += chunk_size as u64;
        let _ = progress_tx.send(InstallProgressEvent::AllFilesOffsetBytes(
            *all_files_offset_bytes,
        ));
    }
    usb_writer.flush()?;

    Ok(())
}

pub fn do_workloop(
    mut usb_reader: impl Read,
    mut usb_writer: impl Write,
    cancel: Option<&AtomicBool>,
    game_paths: &[PathBuf],
    progress_tx: &InstallProgressSender,
) -> color_eyre::Result<()> {
    let mut command_header = [0u8; 0x20];
    let mut read_buf = [0u8; 512];
    let mut stored_game_file = None;
    let mut all_files_offset_bytes = 0;

    loop {
        if cancel.as_ref().is_some_and(|c| c.load(Ordering::Relaxed)) {
            info!("cancellation requested, exiting...");
            return Ok(());
        }

        debug!("waiting for header...");
        usb_reader.read_exact(&mut command_header)?;

        debug!("got header: {:#?}", &command_header);
        if &command_header[..4] != b"TUC0" {
            error!("invalid command header magic. continuing to next iteration...");
            continue;
        }

        debug!("correct command header magic");

        let command_type: [u8; 1] = command_header[4..5].try_into().unwrap();
        let command_id: [u8; 4] = command_header[8..12].try_into().unwrap();
        // let data_size: [u8; 8] = command_header[12..20].try_into().unwrap();

        debug!(
            "Command type: {:?}, Command id: {:?}",
            &command_type, &command_id
        );

        match command_id {
            command::EXIT => {
                debug!("got exit command, exiting...");
                break;
            }
            command::FILE_RANGE => {
                debug!("got file range command");
                file_range_command(
                    &mut usb_reader,
                    &mut usb_writer,
                    &mut read_buf,
                    &mut stored_game_file,
                    game_paths,
                    progress_tx,
                    &mut all_files_offset_bytes,
                )?;
            }
            _ => bail!("invalid tinfoil command encountered!"),
        }
    }
    Ok(())
}

pub fn initiate_transfer(
    usb_writer: &mut impl Write,
    paths_with_newlines_string_length: u32,
    game_paths: &[PathBuf],
) -> color_eyre::Result<()> {
    usb_writer.write_all(b"TUL0")?;
    if let Err(e) = usb_writer.flush()
        && e.kind() == std::io::ErrorKind::TimedOut
    {
        bail!([
            "Failed to perform initial handshake with Sphaira.",
            "Ensure Awoo/CyberFoil is open on the Nintendo Switch",
            "and in the 'Install over USB'/'Install from NS-USBloader' menu.",
            "If you are trying to transfer to Sphaira, you need to change to that USB install option.",
        ].join("\n"));
    }

    usb_writer.write_all(&paths_with_newlines_string_length.to_le_bytes())?;
    usb_writer.flush()?;

    usb_writer.write_all(&[0u8; 8])?;
    usb_writer.flush()?;

    for path in game_paths {
        writeln!(usb_writer, "{}", path.to_str().unwrap())?;
        usb_writer.flush()?;
    }
    Ok(())
}
