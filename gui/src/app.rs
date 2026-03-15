use egui::{
    Align2, Button, ProgressBar, RichText, TextWrapMode,
    Theme::{Dark, Light},
};
use egui_toast::{Toast, ToastKind, Toasts};
use ironfoil_core::{GAME_BACKUP_EXTENSIONS, perform_tinfoil_usb_install};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::mpsc, thread::JoinHandle};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug)]
struct OngoingInstallation {
    progress_len_rx: mpsc::Receiver<u64>,
    progress_rx: mpsc::Receiver<u64>,
    last_progress_len: u64,
    last_progress: u64,
    thread: JoinHandle<color_eyre::Result<()>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct App {
    tab: Tab,
}

#[derive(Serialize, Deserialize, Debug, EnumIter)]
enum Tab {
    Home,
    Usb {
        recurse: bool,
        #[serde(skip)]
        maybe_ongoing_installation: Option<OngoingInstallation>,
    },
    Network,
    Rcm,
}

fn start_usb_install(game_backup_path: PathBuf, recurse: bool) -> OngoingInstallation {
    let (progress_len_tx, progress_len_rx) = mpsc::channel::<u64>();
    let (progress_tx, progress_rx) = mpsc::channel::<u64>();

    OngoingInstallation {
        progress_len_rx,
        progress_rx,
        thread: std::thread::spawn(move || {
            perform_tinfoil_usb_install(&game_backup_path, recurse, progress_len_tx, progress_tx)
        }),
        last_progress: 0,
        last_progress_len: 1,
    }
}

impl Tab {
    fn as_str(&self) -> &'static str {
        match self {
            Tab::Home => "🏠 Home",
            Tab::Usb { .. } => "🔌 USB",
            Tab::Network => "🌐 Network",
            Tab::Rcm => "📎 RCM",
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
                ui.label("Install a game backup from your computer to your Switch using the Tinfoil USB transfer protocol.");
                ui.label("You can either pick a single backup file or a directory containing multiple backups.");
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
                    info!(
                        "progress: {}/{} ({:.2}%)",
                        ongoing_installation.last_progress,
                        ongoing_installation.last_progress_len,
                        progress * 100.
                    );
                    ui.add(ProgressBar::new(progress));

                    // thread is finished? take it!
                    if ongoing_installation.thread.is_finished() {
                        info!("install thread finished");
                        // FIXME: avoid expect. we know that it is Some..
                        let ongoing_installation = maybe_ongoing_installation
                            .take()
                            .expect("there is an ongoing installation");

                        let toast = match ongoing_installation.thread.join() {
                            Ok(Ok(_)) => {
                                info!("installation thread finished with success");
                                Toast {
                                    kind: ToastKind::Success,
                                    text: "Installation completed successfully!".into(),
                                    ..Default::default()
                                }
                            }
                            Ok(Err(e)) => {
                                error!("installation thread finished with error:\n{:?}", e);
                                Toast {
                                    kind: ToastKind::Error,
                                    text: format!("Installation failed:\n{}", e).into(),
                                    ..Default::default()
                                }
                            }
                            Err(e) => {
                                error!("installation thread panicked:\n{:?}", e);
                                Toast {
                                    kind: ToastKind::Error,
                                    text: format!("Installation crashed:\n{:?}", e).into(),
                                    ..Default::default()
                                }
                            }
                        };
                        toasts.add(toast);
                    }
                }
            }
            Tab::Network => {
                // ui.label("Network tab content goes here.");
            }
            Tab::Rcm => {
                // ui.label("RCM tab content goes here.");
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self { tab: Tab::Home }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        if let Some(storage) = cc.storage {
            let stored = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            info!("read from stored! {:?}", &stored);
            stored
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
            let mut toasts = Toasts::new()
                .anchor(Align2::RIGHT_BOTTOM, (-16., -16.))
                .direction(egui::Direction::BottomUp);

            ui.horizontal(|ui| {
                ui.heading(env!("CARGO_PKG_NAME"));
                // if !matches!(self.tab, Tab::Home) {
                //     ui.heading("|");
                //     ui.heading(self.tab.as_str());
                // }
            });
            ui.spacing_mut().item_spacing.y = 8.;
            ui.separator();

            self.tab.show(ui, &ctx.theme(), &mut toasts);

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
            toasts.show(ctx);
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        info!("saving app state: {:?}", self);
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
