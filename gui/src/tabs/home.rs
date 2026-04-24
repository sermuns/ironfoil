use egui::Theme;

pub fn show(ui: &mut egui::Ui, theme: egui::Theme) {
    // FIXME: this relative path shit fucking sucks..
    let banner_source = match theme {
        Theme::Dark => egui::include_image!("../../media/banner-dark.svg"),
        Theme::Light => egui::include_image!("../../media/banner-light.svg"),
    };
    ui.vertical_centered(|ui| {
        ui.add(egui::Image::new(banner_source).max_height(250.));
        ui.label("Select one of the tabs on the left!");
    });
}
