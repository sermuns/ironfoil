use egui::{
    Align, Align2, Button, Checkbox, Color32, Layout, ProgressBar, RichText, TextWrapMode,
    Theme::{self, Dark, Light},
};
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
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug)]
struct OngoingInstallation {
    progress_len_rx: mpsc::Receiver<u64>,
    progress_rx: mpsc::Receiver<u64>,
    last_progress_len: u64,
    last_progress: u64,
    thread: JoinHandle<color_eyre::Result<()>>,
    cancel: Arc<AtomicBool>,
}

#[derive(Default)]
struct StagedFiles {
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
enum Tab {
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
    fn as_str(&self) -> &'static str {
        match self {
            Tab::Home => "🏠 Home",
            Tab::Usb { .. } => "🔌 USB",
            Tab::Network => "🌐 Network",
            Tab::Rcm => "📎 RCM",
            Tab::Log => "📜 Log",
        }
    }

    fn show(&mut self, ui: &mut egui::Ui, theme: &egui::Theme, toasts: &mut Toasts) {
        match self {
            Tab::Home => {
                let banner_source = match theme {
                    Dark => egui::include_image!("../../media/banner-dark.svg"),
                    Light => egui::include_image!("../../media/banner-light.svg"),
                };
                ui.vertical_centered(|ui| {
                    ui.add(egui::Image::new(banner_source).max_height(200.));
                });
                ui.label("Select one of the tabs on the left to get started!");
            }
            Tab::Usb {
                recurse,
                for_sphaira,
                staged_files,
                maybe_ongoing_installation,
            } => {
                ui.label("Install a game backup from your computer to your Nintendo Switch using the Tinfoil USB transfer protocol.");
                ui.label("You can either pick a single backup file or a directory containing multiple backups.");
                ui.label("Check 'Recurse?' if you also want to recursively discover game backups from subdirectories of that directory.");

                ui.add_space(8. * 2.);

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
                    ui.label("and");
                    ui.checkbox(recurse, "recurse?");
                    ui.label("and");
                    ui.checkbox(for_sphaira, "for Sphaira homebrew menu?");
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
                        ui.weak("or");
                        if ui.button("❌ remove from list").clicked() {
                            staged_files.remove_selected();
                        }
                        ui.weak(format!(
                            "The {} selected ({}), I want to...",
                            staged_files.count(),
                            humansize::format_size(staged_files.total_file_size, humansize::BINARY)
                        ));
                    }
                });
            }
            Tab::Network | Tab::Rcm | Tab::Log => {
                ui.label("UNIMPLEMENTED here! Use the command-line tool for now...");
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct App {
    tab: Tab,
    #[serde(skip)]
    toasts: Toasts,
}

impl Default for App {
    fn default() -> Self {
        Self {
            tab: Tab::Home,
            toasts: Toasts::new()
                .anchor(Align2::CENTER_CENTER, egui::Pos2::ZERO)
                .direction(egui::Direction::BottomUp),
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        cc.egui_ctx.style_mut_of(Theme::Light, |style| {
            style.visuals.widgets.noninteractive.fg_stroke.color = Color32::BLACK;
            style.visuals.striped = true;
        });
        cc.egui_ctx.style_mut_of(Theme::Dark, |style| {
            style.visuals.widgets.noninteractive.fg_stroke.color = Color32::WHITE;
            style.visuals.striped = true;
        });

        if let Some(storage) = cc.storage {
            info!("read from stored");
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            info!("no stored");
            Default::default()
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::SidePanel::left("left_panel")
            .resizable(false)
            .show(ctx, |ui| {
                for tab in Tab::iter() {
                    let is_current =
                        std::mem::discriminant(&tab) == std::mem::discriminant(&self.tab);

                    let text = RichText::new(tab.as_str()).size(16.);

                    let response = ui.add_sized(
                        [ui.available_width(), 32.0],
                        Button::selectable(is_current, text)
                            .wrap_mode(TextWrapMode::Extend)
                            .right_text(""),
                    );

                    if response.clicked() {
                        self.tab = tab;
                    }
                }
            });

        egui::TopBottomPanel::bottom("footer")
            .resizable(false)
            .show(ctx, |ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(env!("VERGEN_GIT_DESCRIBE"));
                    ui.hyperlink_to(
                        env!("CARGO_PKG_NAME"),
                        "https://github.com/sermuns/ironfoil",
                    );
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(env!("CARGO_PKG_NAME"));
            ui.separator();
            ui.add_space(8.);
            self.tab.show(ui, &ctx.theme(), &mut self.toasts);
            self.toasts.show(ctx);
        });

        ctx.request_repaint(); // FIXME: unneccessaryily continous.
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
