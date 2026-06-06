use egui::{
    Align, Align2, Button, CentralPanel, Color32, Layout, Panel, RichText, TextWrapMode, Theme,
};
use egui_toast::{Toast, ToastKind, Toasts};
use serde::{Deserialize, Serialize};
use std::{net::Ipv4Addr, time::Duration};
use strum::IntoEnumIterator;

use crate::tabs::Tab;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct App {
    tab: Tab,
    target_ip: Option<Ipv4Addr>,
    #[serde(skip)]
    target_ip_string: String,
    #[serde(skip)]
    toasts: Toasts,
}

impl Default for App {
    fn default() -> Self {
        Self {
            tab: Tab::Home,
            target_ip: None,
            target_ip_string: String::new(),
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

        let Some(storage) = cc.storage else {
            return App::default();
        };
        eframe::get_value(storage, eframe::APP_KEY)
            .map(|mut stored: App| {
                // kinda shitty, but works
                stored.target_ip_string = stored
                    .target_ip
                    .map(|ip| ip.to_string())
                    .unwrap_or_default();
                stored
            })
            .unwrap_or_default()
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        Panel::top("top_panel").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        Panel::left("left_panel")
            .resizable(false)
            .show_inside(ui, |ui| {
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

        Panel::bottom("footer")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(env!("VERGEN_GIT_DESCRIBE"));
                    ui.hyperlink_to(
                        env!("CARGO_PKG_NAME"),
                        "https://github.com/sermuns/ironfoil",
                    );
                });
            });

        CentralPanel::default().show_inside(ui, |ui| {
            self.tab.show(
                ui,
                ui.theme(),
                &mut self.toasts,
                &mut self.target_ip_string,
                &mut self.target_ip,
            );
            self.toasts.show(ui);
        });

        // FIXME:
        ui.request_repaint_after(Duration::from_millis(100));
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

pub fn add_toast(toasts: &mut Toasts, kind: ToastKind, text: impl Into<egui::WidgetText>) {
    toasts.add(Toast {
        kind,
        text: text.into(),
        // .duration_in_seconds(10.)
        // .show_progress(true),
        ..Default::default()
    });
}
