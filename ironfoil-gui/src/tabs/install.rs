use egui::{Align, Checkbox, Color32, ComboBox, Layout, ProgressBar, RichText, TextEdit, Theme};
use egui_extras::{Column, TableBuilder};
use egui_toast::{ToastKind, Toasts};
use ironfoil_core::{
    GAME_BACKUP_EXTENSIONS, InstallProgressEvent, perform_tinfoil_network_install,
    perform_usb_install,
};
use log::{error, info};
use std::{
    net::Ipv4Addr,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};

use crate::tabs::{
    InstallProgress, OngoingInstallation, Pick, StagedFiles, UsbProtocol, stage_picked,
};
use crate::{app::add_toast, tabs::InstallType};

#[allow(clippy::too_many_lines)] // FIXME:
pub fn show(
    ui: &mut egui::Ui,
    theme: egui::Theme,
    recurse: &mut bool,
    install_type: &mut InstallType,
    staged_files: &mut StagedFiles,
    maybe_ongoing_installation: &mut Option<OngoingInstallation>,
    toasts: &mut Toasts,
    target_ip_string: &mut String,
    target_ip: &mut Option<Ipv4Addr>,
) {
    ui.horizontal(|ui| {
        if ui.button("💾 Pick file").clicked()
            && let Some(game_backup_path) = rfd::FileDialog::new()
                .add_filter("Nintendo Switch game backups", &GAME_BACKUP_EXTENSIONS)
                .pick_file()
        {
            stage_picked(Pick::File(game_backup_path), staged_files, toasts);
        }
        ui.weak("or");
        if ui
            .button(if cfg!(target_os = "windows") {
                "🗁 Pick folder"
            } else {
                "🗁 Pick directory"
            })
            .clicked()
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

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if matches!(install_type, InstallType::Network) {
                let mut text_edit = TextEdit::singleline(target_ip_string)
                    .hint_text("IP address")
                    .desired_width(7. * 15.); // random asss. ipv4 addresse should (at most) be 15 characters
                if target_ip.is_none() {
                    text_edit = text_edit.background_color(match theme {
                        Theme::Dark => Color32::DARK_RED,
                        Theme::Light => Color32::LIGHT_RED,
                    });
                }

                if ui.add(text_edit).changed() {
                    *target_ip = target_ip_string.parse().ok();
                }
            }
            ComboBox::from_label(RichText::new("Install type:").weak())
                .selected_text(install_type.as_str())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        install_type,
                        InstallType::Usb {
                            protocol: UsbProtocol::TinFoil,
                        },
                        InstallType::Usb {
                            protocol: UsbProtocol::TinFoil,
                        }
                        .as_str(),
                    );
                    ui.selectable_value(
                        install_type,
                        InstallType::Usb {
                            protocol: UsbProtocol::Sphaira,
                        },
                        InstallType::Usb {
                            protocol: UsbProtocol::Sphaira,
                        }
                        .as_str(),
                    );
                    ui.selectable_value(
                        install_type,
                        InstallType::Network,
                        InstallType::Network.as_str(),
                    );
                });
        });
    });

    ui.group(|ui| {
        if staged_files.is_empty() {
            ui.set_min_size(ui.available_size());
            ui.weak("No files staged for installation. Pick using the buttons above!");
            return;
        }

        // FIXME: so fucking stupid... DONT USE HARDCODE FOR HEIGHT OF OTHER SHIT
        ui.set_height(ui.available_height() - 19. * 2.);

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
                        ui.label(staged_file.path.file_name().unwrap().to_str().unwrap());
                    });
                    row.col(|ui| {
                        ui.label(humansize::format_size(
                            staged_file.file_size,
                            humansize::BINARY,
                        ));
                    });
                });
            });
    });
    // TODO: figure out if we should use `while` or `if`?
    // i guess we dont really need to consume all events, but could we
    // possibly start lagging behidn if we only use if?
    if let Some(ongoing_installation) = maybe_ongoing_installation {
        ui.allocate_ui_with_layout(
            (ui.available_width(), 18.).into(),
            Layout::right_to_left(Align::TOP),
            |ui| {
                if ui.button("❌ cancel").clicked() {
                    ongoing_installation.cancel.store(true, Ordering::Relaxed);
                }
                ui.add(
                    ProgressBar::new(ongoing_installation.progress.all_files_ratio).text(format!(
                        "Total progress | {}/{}",
                        humansize::format_size(
                            ongoing_installation.progress.all_files_offset_bytes,
                            humansize::BINARY,
                        ),
                        humansize::format_size(
                            ongoing_installation.progress.all_files_length_bytes,
                            humansize::BINARY,
                        )
                    )),
                );
            },
        );
        ui.add(
            ProgressBar::new(ongoing_installation.progress.current_file_ratio)
                .show_percentage()
                .text(format!(
                    "{} | {}/{}",
                    ongoing_installation.progress.current_file_name,
                    humansize::format_size(
                        ongoing_installation.progress.current_file_offset_bytes,
                        humansize::BINARY,
                    ),
                    humansize::format_size(
                        ongoing_installation.progress.current_file_length_bytes,
                        humansize::BINARY,
                    )
                )),
        );

        let Ok(progress_event) = ongoing_installation.progress_rx.try_recv() else {
            return;
        };
        match progress_event {
            InstallProgressEvent::CurrentFileName(file_name) => {
                ongoing_installation.progress.current_file_name = file_name;
            }
            InstallProgressEvent::AllFilesLengthBytes(length) => {
                ongoing_installation.progress.all_files_length_bytes = length;
                ongoing_installation.recalculate_all_files_progress();
            }
            InstallProgressEvent::AllFilesOffsetBytes(offset) => {
                ongoing_installation.progress.all_files_offset_bytes = offset;
                ongoing_installation.recalculate_all_files_progress();
            }
            InstallProgressEvent::CurrentFileLengthBytes(length) => {
                ongoing_installation.progress.current_file_length_bytes = length;
                ongoing_installation.recalculate_current_file_progress();
            }
            InstallProgressEvent::CurrentFileOffsetBytes(offset) => {
                ongoing_installation.progress.current_file_offset_bytes = offset;
                ongoing_installation.recalculate_current_file_progress();
            }
            InstallProgressEvent::Ended => {
                ongoing_installation.progress.all_files_ratio = 1.0;
                ongoing_installation.progress.current_file_ratio = 1.0;
                ongoing_installation.progress.ended = true;
            }
        }

        // thread is finished? take it!
        if ongoing_installation.progress.ended {
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
                Ok(Ok(())) => {
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
            }
        }
    } else if !staged_files.is_empty() {
        let game_paths: Vec<_> = staged_files
            .files
            .iter()
            .filter_map(|staged_file| staged_file.selected.then_some(staged_file.path.clone()))
            .collect();

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_enabled_ui(staged_files.has_any_selected(), |ui| {
                if ui
                    .button(match install_type {
                        InstallType::Usb { .. } => "🔌 install over USB!",
                        InstallType::Network => "🖧 install over network!",
                    })
                    .clicked()
                {
                    start_install(
                        game_paths,
                        install_type,
                        *target_ip,
                        maybe_ongoing_installation,
                        toasts,
                    );
                }
                if ui.button("❌ remove selected from stage").clicked() {
                    staged_files.remove_selected();
                }
                ui.weak(format!(
                    "{} selected ({})",
                    staged_files.selected_count(),
                    staged_files.selected_human_size(),
                ));
                ui.weak("of which");
                ui.weak(format!(
                    "{} staged ({}),",
                    staged_files.count(),
                    staged_files.human_size(),
                ));
            })
        });
    }
}

fn start_install(
    game_paths: Vec<PathBuf>,
    install_type: &InstallType,
    target_ip: Option<Ipv4Addr>,
    maybe_ongoing_installation: &mut Option<OngoingInstallation>,
    toasts: &mut Toasts,
) {
    let (progress_tx, progress_rx) = mpsc::channel::<InstallProgressEvent>();

    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_thread = cancel.clone();

    let thread = match install_type {
        InstallType::Usb { protocol } => {
            let protocol = *protocol;
            std::thread::spawn(move || {
                perform_usb_install(&game_paths, progress_tx, protocol, Some(&cancel_thread))
            })
        }
        InstallType::Network => {
            let Some(target_ip) = target_ip else {
                add_toast(
                    toasts,
                    ToastKind::Error,
                    "The given target IP address is not valid!",
                );
                return;
            };
            std::thread::spawn(move || {
                perform_tinfoil_network_install(
                    game_paths,
                    target_ip,
                    progress_tx,
                    Some(cancel_thread),
                )
            })
        }
    };

    *maybe_ongoing_installation = Some(OngoingInstallation {
        progress_rx,
        progress: InstallProgress::default(),
        thread,
        cancel,
    });
}
