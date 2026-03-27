use color_eyre::eyre::{ContextCompat, eyre};
use log::info;
use nusb::{
    MaybeFuture, list_devices,
    transfer::{Bulk, In, Out},
};
use std::{path::PathBuf, sync::atomic::AtomicBool, time::Duration};

use crate::{InstallProgressEvent, InstallProgressSender, UsbProtocol};

mod sphaira;
mod tinfoil;

pub fn perform_usb_install(
    game_paths: &[PathBuf],
    progress_tx: InstallProgressSender,
    usb_protocol: UsbProtocol,
    cancel: Option<&AtomicBool>,
) -> color_eyre::Result<()> {
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
    const USB_WRITER_BUFFER_SIZE_BYTES: usize = 2 ^ 20;
    const USB_WRITER_TIMEOUT: Duration = Duration::from_millis(500);
    let mut usb_writer = ep_out
        .writer(USB_WRITER_BUFFER_SIZE_BYTES)
        .with_write_timeout(USB_WRITER_TIMEOUT);

    let mut ep_in = interface.endpoint::<Bulk, In>(0x81)?;
    ep_in.clear_halt().wait()?;
    const USB_READER_TIMEOUT: Duration = Duration::from_mins(5);
    let mut usb_reader = ep_in.reader(512).with_read_timeout(USB_READER_TIMEOUT);

    let paths_with_newlines_string_length: u32 = game_paths
        .iter()
        .map(|path| path.to_str().unwrap().len() as u32 + 1)
        .sum();

    let all_files_length_bytes = game_paths
        .iter()
        .map(|path| path.metadata().unwrap().len())
        .sum::<u64>();
    let _ = progress_tx.send(InstallProgressEvent::AllFilesLengthBytes(
        all_files_length_bytes,
    ));

    match usb_protocol {
        UsbProtocol::Sphaira => sphaira::initiate_transfer(
            &mut usb_reader,
            &mut usb_writer,
            paths_with_newlines_string_length,
            game_paths,
        )?,

        UsbProtocol::TinFoil => tinfoil::initiate_transfer(
            &mut usb_writer,
            paths_with_newlines_string_length,
            game_paths,
        )?,
    }

    info!("sent list of games to Nintendo Switch.");

    match usb_protocol {
        UsbProtocol::Sphaira => {
            info!("starting Sphaira USB install.");
            sphaira::do_workloop(
                &mut usb_reader,
                &mut usb_writer,
                cancel,
                game_paths,
                &progress_tx,
            )
            .inspect_err(|_| {
                let _ = sphaira::send_result(&mut usb_writer, sphaira::RESULT_ERROR, None, None);
            })?;
        }
        UsbProtocol::TinFoil => {
            info!("starting Tinfoil USB install.");
            tinfoil::do_workloop(usb_reader, usb_writer, cancel, game_paths, &progress_tx)?;
        }
    }

    let _ = progress_tx.send(InstallProgressEvent::Ended);

    Ok(())
}
