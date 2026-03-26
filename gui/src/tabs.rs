use egui::{Align, Checkbox, Layout, ProgressBar, Theme};
use egui_extras::{Column, TableBuilder};
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
use ironfoil_core::{GAME_BACKUP_EXTENSIONS, perform_usb_install};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::JoinHandle,
};
use strum::EnumIter;

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
}

#[derive(Clone)]
struct StagedFile {
    path: PathBuf,
    file_size: u64,
    selected: bool,
}

#[derive(Serialize, Deserialize, EnumIter)]
pub enum Tab {
    Home,
    Usb {
        recurse: bool,
        for_sphaira: bool,
        #[serde(skip)]
        staged_files: StagedFiles,
        #[serde(skip)]
        maybe_ongoing_installation: Option<OngoingInstallation>,
    },
    Network,
    Rcm,
    Log,
}

fn start_usb_install(
    game_paths: Vec<PathBuf>,
    for_sphaira: bool,
    maybe_ongoing_installation: &mut Option<OngoingInstallation>,
) {
    let (progress_len_tx, progress_len_rx) = mpsc::channel::<u64>();
    let (progress_tx, progress_rx) = mpsc::channel::<u64>();

    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_thread = cancel.clone();

    *maybe_ongoing_installation = Some(OngoingInstallation {
        progress_len_rx,
        progress_rx,
        thread: std::thread::spawn(move || {
            perform_usb_install(
                &game_paths,
                progress_len_tx,
                progress_tx,
                for_sphaira,
                cancel_thread,
            )
        }),
        last_progress: 0,
        last_progress_len: 1,
        cancel,
    });
}

fn add_toast(toasts: &mut Toasts, kind: ToastKind, text: impl Into<egui::WidgetText>) {
    toasts.add(Toast {
        kind,
        text: text.into(),
        options: ToastOptions::default(),
        // .duration_in_seconds(10.)
        // .show_progress(true),
        ..Default::default()
    });
}

enum Pick {
    File(PathBuf),
    Folder { path: PathBuf, recurse: bool },
}

fn stage_picked(pick: Pick, staged_files: &mut StagedFiles, toasts: &mut Toasts) {
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
            Tab::Usb { .. } => "🔌 USB",
            Tab::Network => "🌐 Network",
            Tab::Rcm => "📎 RCM",
            Tab::Log => "📜 Log",
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, theme: &egui::Theme, toasts: &mut Toasts) {
        match self {
            Tab::Home => {
                let banner_source = match theme {
                    Theme::Dark => egui::include_image!("../../media/banner-dark.svg"),
                    Theme::Light => egui::include_image!("../../media/banner-light.svg"),
                };
                ui.vertical_centered(|ui| {
                    ui.add(egui::Image::new(banner_source).max_height(200.));
                });
                ui.label("Select one of the tabs on the left!");
            }
            Tab::Usb {
                recurse,
                for_sphaira,
                staged_files,
                maybe_ongoing_installation,
            } => {
                ui.horizontal(|ui| {
                    if ui.button("💾 Pick file").clicked()
                        && let Some(game_backup_path) = rfd::FileDialog::new()
                            .add_filter("*", &GAME_BACKUP_EXTENSIONS)
                            .pick_file()
                    {
                        stage_picked(Pick::File(game_backup_path), staged_files, toasts);
                    }
                    ui.label("or");
                    #[cfg(target_os = "windows")]
                    const PICK_DIRECTORY_LABEL: &str = "🗁 Pick folder";
                    #[cfg(not(target_os = "windows"))]
                    const PICK_DIRECTORY_LABEL: &str = "🗁 Pick directory";
                    if ui.button(PICK_DIRECTORY_LABEL).clicked()
                        && let Some(game_backup_path) = rfd::FileDialog::new().pick_folder()
                    {
                        stage_picked(
                            Pick::Folder {
                                path: game_backup_path,
                                recurse: *recurse,
                            },
                            staged_files,
                            toasts,
                        );
                    }
                    ui.checkbox(recurse, "recurse?").on_hover_text(
                        "Also discover game backups from subdirectories of the picked directory",
                    );

                    // FIXME: actually align right
                    ui.add_space(16.);
                    ui.checkbox(for_sphaira, "For Sphaira?");
                });

                ui.group(|ui| {
                    if staged_files.is_empty() {
                        ui.set_min_size(ui.available_size());
                        ui.weak("No files staged. Pick using the buttons above!");
                        return;
                    }

                    // FIXME: so fucking stupid... DONT USE HARDCODE FOR HEIGHT OF OTHER SHIT
                    ui.set_height(ui.available_height() - 18. * 2.);

                    if staged_files.has_any_selected() {
                        if ui.button("Deselect all").clicked() {
                            staged_files.deselect_all();
                        }
                    } else if ui.button("Select all").clicked() {
                        staged_files.select_all();
                    }

                    TableBuilder::new(ui)
                        .column(Column::auto())
                        .column(Column::remainder())
                        .column(Column::auto())
                        .header(0., |mut header| {
                            header.col(|ui| {
                                ui.strong("Selected");
                            });
                            header.col(|ui| {
                                ui.strong("File name");
                            });
                            header.col(|ui| {
                                ui.strong("Size");
                            });
                        })
                        .body(|body| {
                            body.rows(18., staged_files.count(), |mut row| {
                                let staged_file = &mut staged_files.files[row.index()];
                                row.col(|ui| {
                                    ui.add(Checkbox::without_text(&mut staged_file.selected));
                                });
                                row.col(|ui| {
                                    ui.label(
                                        staged_file.path.file_name().unwrap().to_str().unwrap(),
                                    );
                                });
                                row.col(|ui| {
                                    ui.label(humansize::format_size(
                                        staged_file.file_size,
                                        humansize::BINARY,
                                    ));
                                });
                            })
                        });
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if let Some(ongoing_installation) = maybe_ongoing_installation {
                        if let Ok(progress_len) = ongoing_installation.progress_len_rx.try_recv() {
                            info!("got progress len: {}", progress_len);
                            ongoing_installation.last_progress_len = progress_len;
                        }
                        if let Ok(progress) = ongoing_installation.progress_rx.try_recv() {
                            info!("got progress: {}", progress);
                            ongoing_installation.last_progress = progress;
                        }
                        let progress: f32 = ongoing_installation.last_progress as f32
                            / ongoing_installation.last_progress_len as f32;
                        info!(
                            "progress: {}/{} ({:.2}%)",
                            ongoing_installation.last_progress,
                            ongoing_installation.last_progress_len,
                            progress * 100.
                        );
                        ui.horizontal(|ui| {
                            if ui.button("❌ cancel").clicked() {
                                ongoing_installation.cancel.store(true, Ordering::Relaxed);
                            }
                            ui.add(ProgressBar::new(progress));
                        });

                        // thread is finished? take it!
                        if ongoing_installation.thread.is_finished() {
                            info!("install thread finished");
                            // FIXME: avoid expect. we know that it is Some..
                            let ongoing_installation = maybe_ongoing_installation
                                .take()
                                .expect("there is an ongoing installation");

                            if ongoing_installation.cancel.load(Ordering::Relaxed) {
                                info!("installation was cancelled");
                                add_toast(toasts, ToastKind::Info, "Installation cancelled.");
                                return;
                            }

                            match ongoing_installation.thread.join() {
                                Ok(Ok(_)) => {
                                    info!("installation thread finished with success");
                                    add_toast(
                                        toasts,
                                        ToastKind::Success,
                                        "Installation completed successfully!",
                                    );
                                }
                                Ok(Err(e)) => {
                                    error!("installation thread finished with error:\n{:?}", e);
                                    add_toast(
                                        toasts,
                                        ToastKind::Error,
                                        format!("Installation failed:\n{}", e),
                                    );
                                }
                                Err(e) => {
                                    error!("installation thread panicked:\n{:?}", e);
                                    add_toast(
                                        toasts,
                                        ToastKind::Error,
                                        format!("Installation crashed:\n{:?}", e),
                                    );
                                }
                            };
                        }
                    } else if !staged_files.is_empty() {
                        if ui.button("🔌 install now!").clicked() {
                            let game_paths: Vec<_> = staged_files
                                .files
                                .iter()
                                .filter_map(|staged_file| {
                                    staged_file.selected.then_some(staged_file.path.clone())
                                })
                                .collect();

                            start_usb_install(game_paths, *for_sphaira, maybe_ongoing_installation);
                        }
                        if ui.button("❌ remove from list").clicked() {
                            staged_files.remove_selected();
                        }
                        // FIXME: fuckjing horrible
                        ui.weak(format!(
                            "{} selected ({})",
                            staged_files
                                .files
                                .iter()
                                .filter(|staged_file| staged_file.selected)
                                .count(),
                            humansize::format_size(
                                staged_files
                                    .files
                                    .iter()
                                    .filter(|staged_file| staged_file.selected)
                                    .map(|staged_file| staged_file.file_size)
                                    .sum::<u64>(),
                                humansize::BINARY
                            ),
                        ));
                    }
                });
            }
            Tab::Network | Tab::Rcm => {
                ui.label("UNIMPLEMENTED here! Use the command-line version for now...");
            }
            Tab::Log => {
                ui.label("UNIMPLEMENTED..! one day it will not be.");
            }
        }
    }
}
