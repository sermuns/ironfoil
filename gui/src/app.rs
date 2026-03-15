use egui::{
    Align2, Button, ProgressBar, RichText, TextWrapMode,
    Theme::{Dark, Light},
};
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
use ironfoil_core::{GAME_BACKUP_EXTENSIONS, perform_tinfoil_usb_install};
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

#[derive(Serialize, Deserialize, EnumIter)]
enum Tab {
    Home,
    Usb {
        recurse: bool,
        #[serde(skip)]
        maybe_ongoing_installation: Option<OngoingInstallation>,
    },
    Network,
    Rcm,
    Log,
}

fn start_usb_install(game_backup_path: PathBuf, recurse: bool) -> OngoingInstallation {
    let (progress_len_tx, progress_len_rx) = mpsc::channel::<u64>();
    let (progress_tx, progress_rx) = mpsc::channel::<u64>();

    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_thread = cancel.clone();

    OngoingInstallation {
        progress_len_rx,
        progress_rx,
        thread: std::thread::spawn(move || {
            perform_tinfoil_usb_install(
                &game_backup_path,
                recurse,
                progress_len_tx,
                progress_tx,
                cancel_thread,
            )
        }),
        last_progress: 0,
        last_progress_len: 1,
        cancel,
    }
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
                maybe_ongoing_installation,
            } => {
                ui.label("Install a game backup from your computer to your Nintendo Switch using the Tinfoil USB transfer protocol.");
                ui.label("You can either pick a single backup file or a directory containing multiple backups.");
                ui.label("Check 'Recurse?' if you also want to recursively discover game backups from subdirectories of that directory.");
                if ui.button("Pick file").clicked()
                    && let Some(game_backup_path) = rfd::FileDialog::new()
                        .add_filter("*", &GAME_BACKUP_EXTENSIONS)
                        .pick_file()
                {
                    *maybe_ongoing_installation =
                        Some(start_usb_install(game_backup_path, *recurse));
                }

                ui.horizontal(|ui| {
                    if ui.button("Pick directory").clicked()
                        && let Some(game_backup_path) = rfd::FileDialog::new().pick_folder()
                    {
                        *maybe_ongoing_installation =
                            Some(start_usb_install(game_backup_path, *recurse));
                    }
                    ui.checkbox(recurse, "Recurse?");
                });

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
                    ui.horizontal(|ui| {
                        ui.add(ProgressBar::new(progress));
                    });

                    if ui.button("Cancel").clicked() {
                        ongoing_installation.cancel.store(true, Ordering::Relaxed);
                    }

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
                }
            }
            Tab::Network => {
                ui.label("UNIMPLEMENTED!");
            }
            Tab::Rcm => {
                ui.label("UNIMPLEMENTED!");
            }
            Tab::Log => {
                ui.label("UNIMPLEMENTED!");
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
                    let selected =
                        std::mem::discriminant(&tab) == std::mem::discriminant(&self.tab);

                    let text = RichText::new(tab.as_str()).size(16.);

                    let response = ui.add_sized(
                        [ui.available_width(), 32.0],
                        Button::selectable(selected, text)
                            .wrap_mode(TextWrapMode::Extend)
                            .right_text(""),
                    );

                    if response.clicked() {
                        self.tab = tab;
                    }
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(env!("CARGO_PKG_NAME"));
                // if !matches!(self.tab, Tab::Home) {
                //     ui.heading("|");
                //     ui.heading(self.tab.as_str());
                // }
            });
            ui.spacing_mut().item_spacing.y = 8.;
            ui.separator();

            self.tab.show(ui, &ctx.theme(), &mut self.toasts);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                ui.horizontal(|ui| {
                    ui.label(env!("VERGEN_GIT_DESCRIBE"));
                    ui.hyperlink_to(
                        env!("CARGO_PKG_NAME"),
                        "https://github.com/sermuns/ironfoil",
                    );
                });
                ui.separator();
            });

            self.toasts.show(ctx);
        });
        ctx.request_repaint(); // FIXME: unneccessaryily continous.
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
