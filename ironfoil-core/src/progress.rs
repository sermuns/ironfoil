use std::sync::mpsc;

#[derive(Debug)]
pub enum InstallProgressEvent {
    /// Request to show status message to user.
    CurrentFileName(String),
    /// Installation has ended, either successfully or by error. Check thread return value!
    Ended,
    /// Total size of all requested files. Should only be sent once on install start.
    AllFilesLengthBytes(u64),
    /// How far we've gotten through all files, totally.
    AllFilesOffsetBytes(u64),
    /// The size of current file being installed. Should only be sent on start of this file.
    CurrentFileLengthBytes(u64),
    /// How far we've gotten through current file.
    CurrentFileOffsetBytes(u64),
}
pub type InstallProgressSender = mpsc::Sender<InstallProgressEvent>;
pub type InstallProgressReceiver = mpsc::Receiver<InstallProgressEvent>;

pub struct InstallEndGuard<'a> {
    pub tx: &'a InstallProgressSender,
}
impl Drop for InstallEndGuard<'_> {
    fn drop(&mut self) {
        let _ = self.tx.send(InstallProgressEvent::Ended);
    }
}
