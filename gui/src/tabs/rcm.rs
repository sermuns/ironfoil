use egui_toast::{ToastKind, Toasts};
use ironfoil_core::RCM_PAYLOAD_EXTENSIONS;
use log::{error, info};
use std::path::PathBuf;

use crate::app::add_toast;

pub fn show(ui: &mut egui::Ui, payload_path: &mut Option<PathBuf>, toasts: &mut Toasts) {
    if ui.button("📦 Pick payload from file").clicked() {
        *payload_path = rfd::FileDialog::new()
            .add_filter("RCM payloads", &RCM_PAYLOAD_EXTENSIONS)
            .pick_file();
    }

    ui.group(|ui| {
        ui.take_available_space();
        let Some(payload_path) = payload_path else {
            ui.weak("No payload selected");
            return;
        };
        ui.weak(payload_path.display().to_string());

        if ui.button("🔌 Send payload").clicked() {
            match ironfoil_core::send_rcm_payload(payload_path) {
                Ok(()) => {
                    info!("successfully sent RCM payload '{}'", payload_path.display());
                    add_toast(
                        toasts,
                        ToastKind::Success,
                        format!("Successfully sent RCM payload '{}'", payload_path.display()),
                    );
                }
                Err(e) => {
                    error!("error while sending RCM payload:\n{:?}", e);
                    add_toast(
                        toasts,
                        ToastKind::Error,
                        format!(
                            "Error while sending RCM payload '{}':\n{:?}",
                            payload_path.display(),
                            e
                        ),
                    );
                }
            }
        }
    });
}
