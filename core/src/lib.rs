mod network;
mod paths;
mod rcm;
mod usb;

#[derive(Debug)]
pub enum InstallProgressEvent {
    Message(String),
    TotalLengthBytes(u64),
    TotalOffsetBytes(u64),
    FileLengthBytes(u64),
    FileOffsetBytes(u64),
}
pub type InstallProgressSender = std::sync::mpsc::Sender<InstallProgressEvent>;
pub type InstallProgressReceiver = std::sync::mpsc::Receiver<InstallProgressEvent>;

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum UsbProtocol {
    TinFoil,
    Sphaira,
}

pub use network::perform_tinfoil_network_install;
pub use paths::{GAME_BACKUP_EXTENSIONS, RCM_PAYLOAD_EXTENSIONS, read_game_paths};
pub use rcm::send_rcm_payload;
pub use usb::perform_usb_install;
