use std::path::PathBuf;

use egui::{
    Align2, Button, RichText,
    Theme::{Dark, Light},
};
use egui_toast::{Toast, ToastKind, Toasts};
use ironfoil_core::{GAME_BACKUP_EXTENSIONS, perform_tinfoil_usb_install};
use log::{error, info};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct App {
    tab: Tab,
}

#[derive(Serialize, Deserialize, Debug, EnumIter, PartialEq)]
enum Tab {
    Home,
    Usb { recurse: bool },
    Network,
    Rcm,
}

fn try_install(path: Option<PathBuf>, recurse: bool, toasts: &mut Toasts) {
    if let Some(path) = path
        && let Err(e) = perform_tinfoil_usb_install(&path, recurse)
    {
        error!("{}", e);
        toasts.add(Toast {
            kind: ToastKind::Error,
            text: e.to_string().into(),
            ..Default::default()
        });
    }
}

impl Tab {
    fn as_str(&self) -> &'static str {
        match self {
            Tab::Home => "Home",
            Tab::Usb { .. } => "USB",
            Tab::Network => "Network",
            Tab::Rcm => "RCM",
        }
    }

    fn show(
        &mut self,
        ui: &mut egui::Ui,
        theme: &egui::Theme,
        toasts: &mut Toasts,
    ) -> egui::Response {
        match self {
            Tab::Home => {
                ui.label("Select one of the tabs on the left to get started!");
                match theme {
                    Dark => ui.image(egui::include_image!("../../media/banner-dark.svg")),
                    Light => ui.image(egui::include_image!("../../media/banner-light.svg")),
                }
            }
            Tab::Usb { recurse } => {
                ui.add_space(4.);
                if ui.button("Install from file").clicked() {
                    try_install(
                        rfd::FileDialog::new()
                            .add_filter("*", &GAME_BACKUP_EXTENSIONS)
                            .pick_file(),
                        *recurse,
                        toasts,
                    );
                }

                ui.horizontal(|ui| {
                    if ui.button("Install from folder").clicked() {
                        try_install(rfd::FileDialog::new().pick_folder(), *recurse, toasts);
                    }
                    ui.checkbox(recurse, "Recurse?");
                });

                ui.label("he")
            }
            Tab::Network => ui.label("Network tab content goes here."),
            Tab::Rcm => ui.label("RCM tab content goes here."),
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
                    if self.tab == tab {
                        if ui.button(RichText::new(tab.as_str()).strong()).clicked() {
                            self.tab = tab;
                        }
                    } else if ui.button(tab.as_str()).clicked() {
                        self.tab = tab;
                    }
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut toasts = Toasts::new()
                .anchor(Align2::RIGHT_BOTTOM, (-16., -16.))
                .direction(egui::Direction::BottomUp);

            ui.horizontal(|ui| {
                if !matches!(self.tab, Tab::Home) {
                    ui.heading(self.tab.as_str());
                    ui.heading("|");
                }
                ui.heading(env!("CARGO_PKG_NAME"));
            });
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
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
