use color_eyre::eyre::bail;
use log::{debug, error, info};
use nusb::{
    Endpoint,
    transfer::{Buffer, Bulk, In, Out},
};
use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use crate::{
    InstallProgressEvent, InstallProgressSender,
    usb::{read_usb, write_usb},
};

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
    write_usb(ep_out, command_direction::RESPONSE)?;
    write_usb(ep_out, command::FILE_RANGE)?;
    write_usb(ep_out, range_size.to_le_bytes())?;
    write_usb(ep_out, [0u8; 0xC])?; // padding?
    Ok(())
}

pub fn file_range_command(
    ep_in: &mut Endpoint<Bulk, In>,
    ep_out: &mut Endpoint<Bulk, Out>,
    game_paths: &[PathBuf],
    progress_tx: &InstallProgressSender,
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
    }

    let _ = progress_tx.send(InstallProgressEvent::Message(game_path.to_string()));

    info!(
        "Range size: {}, Range offset: {}, Name len: {}, Name: {}",
        range_size, range_offset, game_path_len, game_path,
    );

    send_response_header(ep_out, range_size)?;

    let file = File::open(game_path)?;

    if let Ok(metadata) = file.metadata() {
        let file_size = metadata.len();
        let _ = progress_tx.send(InstallProgressEvent::TotalLengthBytes(file_size));
    }

    let mut reader = BufReader::new(file);

    reader.seek(SeekFrom::Start(range_offset))?;
    let _ = progress_tx.send(InstallProgressEvent::TotalOffsetBytes(range_offset));
    let _ = progress_tx.send(InstallProgressEvent::FileLengthBytes(range_size as u64));

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
        progress_tx.send(InstallProgressEvent::FileOffsetBytes(current_offset as u64))?;
    }

    Ok(())
}

pub fn do_workloop(
    ep_in: &mut Endpoint<Bulk, In>,
    ep_out: &mut Endpoint<Bulk, Out>,
    cancel: Option<&AtomicBool>,
    game_paths: &[PathBuf],
    progress_tx: InstallProgressSender,
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
            command::EXIT => {
                debug!("got exit command, exiting...");
                break;
            }
            command::FILE_RANGE => {
                debug!("got file range command");
                file_range_command(ep_in, ep_out, game_paths, &progress_tx)?;
            }
            _ => bail!("invalid tinfoil command encountered!"),
        }
    }
    Ok(())
}
