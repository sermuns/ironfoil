use egui_toast::{ToastKind, Toasts};
use log::error;
use serde::{Deserialize, Serialize};
use std::{
    net::Ipv4Addr,
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool, mpsc},
    thread::JoinHandle,
};
use strum::EnumIter;

use crate::app::add_toast;

mod home;
mod install;
mod rcm;

#[derive(Serialize, Deserialize, EnumIter)]
pub enum Tab {
    Home,
    Install {
        recurse: bool,
        install_type: InstallType,
        #[serde(skip)]
        staged_files: StagedFiles,
        #[serde(skip)]
        maybe_ongoing_installation: Option<OngoingInstallation>,
    },
    Rcm {
        payload_path: Option<PathBuf>,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum UsbProtocol {
    TinFoil,
    Sphaira,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum InstallType {
    Usb { protocol: UsbProtocol },
    Network,
}

impl InstallType {
    pub fn as_str(&self) -> &str {
        match self {
            InstallType::Usb {
                protocol: UsbProtocol::TinFoil,
            } => "🔌 USB (Awoo, CyberFoil, etc.)",
            InstallType::Usb {
                protocol: UsbProtocol::Sphaira,
            } => "🔌 USB (Sphaira)",
            InstallType::Network => "🖧 Network",
        }
    }
}

impl Default for InstallType {
    fn default() -> Self {
        Self::Usb {
            protocol: UsbProtocol::TinFoil,
        }
    }
}

#[derive(Debug)]
pub struct OngoingInstallation {
    progress_len_rx: mpsc::Receiver<u64>,
    progress_rx: mpsc::Receiver<u64>,
    last_progress_len: u64,
    last_progress: u64,
    thread: JoinHandle<color_eyre::Result<()>>,
    cancel: Arc<AtomicBool>,
}

#[derive(Default)]
pub struct StagedFiles {
    files: Vec<StagedFile>,
    total_file_size: u64,
}

impl StagedFiles {
    fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    fn has_any_selected(&self) -> bool {
        self.files.iter().any(|staged_file| staged_file.selected)
    }

    fn deselect_all(&mut self) {
        for staged_file in &mut self.files {
            staged_file.selected = false;
        }
    }
    fn select_all(&mut self) {
        for staged_file in &mut self.files {
            staged_file.selected = true;
        }
    }

    /// add only unique paths to staged files
    fn extend_unique(&mut self, new_paths: Vec<PathBuf>) {
        for path in new_paths {
            if self.contains(&path) {
                continue;
            }
            let file_size = path.metadata().unwrap().len();
            self.total_file_size += file_size;
            self.files.push(StagedFile {
                path,
                file_size,
                selected: true,
            });
        }
    }

    fn contains(&self, path: &PathBuf) -> bool {
        self.files
            .iter()
            .any(|staged_file| &staged_file.path == path)
    }

    fn remove_selected(&mut self) {
        self.files.retain(|staged_file| !staged_file.selected);
        self.total_file_size = self
            .files
            .iter()
            .map(|staged_file| staged_file.file_size)
            .sum();
    }

    fn count(&self) -> usize {
        self.files.len()
    }

    fn selected_count(&self) -> usize {
        self.files
            .iter()
            .filter(|staged_file| staged_file.selected)
            .count()
    }

    fn selected_human_size(&self) -> String {
        let selected_size: u64 = self
            .files
            .iter()
            .filter(|staged_file| staged_file.selected)
            .map(|staged_file| staged_file.file_size)
            .sum();
        humansize::format_size(selected_size, humansize::BINARY)
    }
}

#[derive(Clone)]
struct StagedFile {
    path: PathBuf,
    file_size: u64,
    selected: bool,
}

pub enum Pick {
    File(PathBuf),
    Folder { path: PathBuf, recurse: bool },
}

pub fn stage_picked(pick: Pick, staged_files: &mut StagedFiles, toasts: &mut Toasts) {
    // FIXME: shitty intermediate Vec!!!
    let game_paths = match pick {
        Pick::File(game_path) => vec![game_path],
        Pick::Folder { path, recurse } => match ironfoil_core::read_game_paths(&path, recurse) {
            Ok(game_paths) => game_paths,
            Err(e) => {
                error!("error while reading game paths:\n{:?}", e);
                add_toast(toasts, ToastKind::Error, e.to_string());
                return;
            }
        },
    };

    staged_files.extend_unique(game_paths);
}

impl Tab {
    pub fn as_str(&self) -> &'static str {
        match self {
            Tab::Home => "🏠 Home",
            Tab::Install { .. } => "📥 Install",
            Tab::Rcm { .. } => "📎 RCM",
            // Tab::Log => "📜 Log",
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        theme: egui::Theme,
        toasts: &mut Toasts,
        target_ip_string: &mut String,
        target_ip: &mut Option<Ipv4Addr>,
    ) {
        match self {
            Tab::Home => home::show(ui, theme),
            Tab::Install {
                recurse,
                install_type,
                staged_files,
                maybe_ongoing_installation,
            } => install::show(
                ui,
                theme,
                recurse,
                install_type,
                staged_files,
                maybe_ongoing_installation,
                toasts,
                target_ip_string,
                target_ip,
            ),
            Tab::Rcm { payload_path } => rcm::show(ui, payload_path, toasts),
        }
    }
}
