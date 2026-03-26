use egui::{Align, Align2, Button, Color32, Layout, RichText, TextWrapMode, Theme};
use egui_toast::Toasts;
use log::info;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::tabs::Tab;

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
