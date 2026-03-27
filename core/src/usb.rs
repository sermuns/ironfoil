use color_eyre::eyre::{Context, ContextCompat, eyre};
use log::info;
use nusb::{
    Endpoint, MaybeFuture, list_devices,
    transfer::{Buffer, Bulk, In, Out, TransferError},
};
use std::{path::PathBuf, sync::atomic::AtomicBool, time::Duration};

use crate::{InstallProgressSender, UsbProtocol};

mod sphaira;
mod tinfoil;

pub const USB_TIMEOUT: Duration = Duration::from_millis(500);

pub fn perform_usb_install(
    game_paths: &[PathBuf],
    progress_tx: InstallProgressSender,
    usb_protocol: UsbProtocol,
    cancel: Option<&AtomicBool>,
) -> color_eyre::Result<()> {
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

    match usb_protocol {
        UsbProtocol::Sphaira => {
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
        }
        UsbProtocol::TinFoil => {
            write_usb(&mut ep_out, "TUL0")?;
            write_usb(&mut ep_out, paths_with_newlines_string_length.to_le_bytes())?;
            write_usb(&mut ep_out, [0u8; 8])?;
            for path in game_paths {
                write_usb(&mut ep_out, [path.to_str().unwrap(), "\n"].concat())?;
            }
        }
    }

    eprintln!("Sent list of games to Nintendo Switch.");

    match usb_protocol {
        UsbProtocol::Sphaira => {
            eprintln!("Starting Sphaira USB install.");
            sphaira::do_workloop(&mut ep_in, &mut ep_out, cancel, game_paths, progress_tx)
                .inspect_err(|_| {
                    let _ = sphaira::send_result(&mut ep_out, sphaira::RESULT_ERROR, None, None);
                })?;
        }
        UsbProtocol::TinFoil => {
            eprintln!("Starting tinfoil USB install.");
            tinfoil::do_workloop(&mut ep_in, &mut ep_out, cancel, game_paths, progress_tx)?;
        }
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
