use std::path::PathBuf;

use super::style;
use crate::command::CommandEvent;
use crate::utils;
use crate::utils::dir::FeEntry;
use egui::{Response, RichText, Ui};
use egui_extras::TableBody;

pub fn draw_back_dir_row(
    body: &mut TableBody,
    current_path: PathBuf,
    row_height: f32,
    style: &style::Style,
) -> Option<CommandEvent> {
    let entry = FeEntry {
        name: "..".into(),
        path: current_path.parent().unwrap().to_path_buf(),
        entry_type: utils::dir::EntryKind::Dir(utils::dir::Dir {}),
    };
    return draw_file_row(body, &entry, row_height, style);
}

pub fn draw_file_row(
    body: &mut TableBody,
    entry: &FeEntry,
    row_height: f32,
    style: &style::Style,
) -> Option<CommandEvent> {
    let mut ret = None;
    body.row(row_height, |mut row| {
        row.col(|ui| match draw_file_name_cell(ui, &entry, style) {
            Some(cmd) => ret = Some(cmd),
            None => (),
        });
        row.col(|ui| match draw_file_size_cell(ui, &entry) {
            Some(cmd) => ret = Some(cmd),
            None => (),
        });
        row.col(|ui| match draw_last_modified_cell(ui, &entry) {
            Some(cmd) => ret = Some(cmd),
            None => (),
        });
    });

    return ret;
}

pub fn draw_file_name_cell(
    ui: &mut egui::Ui,
    entry: &FeEntry,
    style: &style::Style,
) -> Option<CommandEvent> {
    let mut ret = None;

    let name = entry.name.to_owned().to_str().unwrap().to_owned();
    let icon = match entry.entry_type {
        utils::dir::EntryKind::Dir(_) => "📁",
        utils::dir::EntryKind::File(_) => "📃",
    };

    cell(ui, |ui| {
        ui.label(icon);
        match &entry.entry_type {
            utils::dir::EntryKind::Dir(_) => {
                let link = ui.link(name);
                link.context_menu(|ui| match get_file_context_menu(ui, entry) {
                    Some(cmd) => ret = Some(cmd),
                    None => (),
                });
                if link.clicked() {
                    ret = Some(CommandEvent::SetPath(entry.path.clone()));
                    println!("ret = {:?}", ret);
                }
            }
            utils::dir::EntryKind::File(file) => {
                let resp = if file.is_exe {
                    let exe = ui.link(RichText::new(name).color(style.colors.exe));
                    if exe.clicked() {
                        ret = Some(CommandEvent::Run(entry.path.clone()));
                    }
                    exe
                } else {
                    ui.label(name)
                };
                resp.context_menu(|ui| {
                    match get_file_context_menu(ui, entry) {
                        Some(cmd) => ret = Some(cmd),
                        None => (),
                    };
                });
            }
        }
        ui.allocate_space(ui.available_size());
    })
    .context_menu(|ui| {
        match get_file_context_menu(ui, entry) {
            Some(cmd) => ret = Some(cmd),
            None => (),
        };
    });

    return ret;
}

pub fn draw_file_size_cell(ui: &mut egui::Ui, entry: &FeEntry) -> Option<CommandEvent> {
    let mut ret = None;

    cell(ui, |ui| {
        match &entry.entry_type {
            utils::dir::EntryKind::Dir(_) => {
                ui.label("");
            }
            utils::dir::EntryKind::File(file) => {
                ui.label(utils::human_readable_size(file.size).to_string())
                    .context_menu(|ui| {
                        match get_file_context_menu(ui, entry) {
                            Some(cmd) => ret = Some(cmd),
                            None => (),
                        };
                    });
            }
        }
        ui.allocate_space(ui.available_size());
    })
    .context_menu(|ui| {
        match get_file_context_menu(ui, entry) {
            Some(cmd) => {
                ret = Some(cmd);
            }
            None => (),
        };
    });

    return ret;
}

pub fn draw_last_modified_cell(ui: &mut egui::Ui, entry: &FeEntry) -> Option<CommandEvent> {
    let mut ret = None;

    cell(ui, |ui| {
        match &entry.entry_type {
            utils::dir::EntryKind::Dir(_) => {
                ui.label("");
            }
            utils::dir::EntryKind::File(file) => {
                ui.label(utils::system_time_to_human_readable(file.modified))
                    .context_menu(|ui| {
                        match get_file_context_menu(ui, entry) {
                            Some(cmd) => ret = Some(cmd),
                            None => (),
                        };
                    });
            }
        }
        ui.allocate_space(ui.available_size());
    })
    .context_menu(|ui| {
        match get_file_context_menu(ui, entry) {
            Some(cmd) => {
                ret = Some(cmd);
            }
            None => (),
        };
    });

    return ret;
}

pub fn get_file_context_menu(ui: &mut Ui, entry: &FeEntry) -> Option<CommandEvent> {
    let mut ret = None;
    match &entry.entry_type {
        utils::dir::EntryKind::Dir(_) => {
            if ui.button("Open").clicked() {
                ui.close_menu();
                ret = Some(CommandEvent::SetPath(entry.path.clone()));
            }
        }
        utils::dir::EntryKind::File(file) => {
            if file.is_exe {
                if ui.button("Run").clicked() {
                    ui.close_menu();
                    ret = Some(CommandEvent::Run(entry.path.clone()));
                }
            }
        }
    }

    if ui.button("Properties").clicked() {
        ui.close_menu();
        // TODO
    }
    if ui.button("Copy path").clicked() {
        ui.ctx()
            .output_mut(|o| o.copied_text = entry.path.to_string_lossy().to_string());
        ui.close_menu();
    }
    if ui.button("Delete").clicked() {
        ret = Some(CommandEvent::DeleteFile(entry.clone()));
    }
    if ui.button("Rename").clicked() {
        println!("TODO rename file");
    }
    ui.separator();
    match get_current_dir_context_menu(ui) {
        Some(cmd) => ret = Some(cmd),
        None => (),
    };

    if ret.is_some() {
        ui.close_menu();
    }

    return ret;
}

// context menu for the dir currently being browsed
pub fn get_current_dir_context_menu(ui: &mut Ui) -> Option<CommandEvent> {
    let mut ret = None;
    if ui.button("New File").clicked() {
        ret = Some(CommandEvent::NewFile)
    }
    if ui.button("Open Terminal").clicked() {
        ret = Some(CommandEvent::OpenTerminal)
    }

    if ret.is_some() {
        ui.close_menu();
    }

    return ret;
}

pub fn cell<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> Response {
    ui.horizontal(add_contents).response
}
