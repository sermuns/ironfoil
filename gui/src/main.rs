#![windows_subsystem = "windows"]

mod app;
mod tabs;

fn main() -> eframe::Result {
    env_logger::init();

    let icon_image = image::load_from_memory(include_bytes!("../media/icon-dark.png")).unwrap();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_icon(std::sync::Arc::new(egui::IconData {
            rgba: icon_image.to_rgba8().to_vec(),
            width: icon_image.width(),
            height: icon_image.height(),
        })),
        ..Default::default()
    };

    eframe::run_native(
        env!("CARGO_PKG_NAME"),
        native_options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
