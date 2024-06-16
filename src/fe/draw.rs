use std::path::PathBuf;

use super::style;
use crate::commands::Commands;
use crate::events::{EventPool, EventType};
use crate::utils;
use crate::utils::dir::FeEntry;
use egui::{Response, RichText, Ui};
use egui_extras::TableBody;

pub fn draw_back_dir_row(
    body: &mut TableBody,
    current_path: PathBuf,
    row_height: f32,
    style: &style::Style,
    event_pool: &mut EventPool,
    commands: &Commands,
) -> Option<()> {
    let path = current_path.parent()?.to_path_buf();
    let entry = FeEntry {
        name: "..".into(),
        path,
        entry_type: utils::dir::EntryKind::Dir(utils::dir::Dir {}),
    };
    draw_file_row(body, &entry, row_height, style, event_pool, commands);
    return Some(());
}

pub fn draw_file_row(
    body: &mut TableBody,
    entry: &FeEntry,
    row_height: f32,
    style: &style::Style,
    event_pool: &mut EventPool,
    commands: &Commands,
) {
    body.row(row_height, |mut row| {
        row.col(|ui| draw_file_name_cell(ui, &entry, style, event_pool, commands));
        row.col(|ui| draw_file_size_cell(ui, &entry, event_pool, commands));
        row.col(|ui| draw_last_modified_cell(ui, &entry, event_pool, commands));
    });
}

pub fn draw_file_name_cell(
    ui: &mut egui::Ui,
    entry: &FeEntry,
    style: &style::Style,
    event_pool: &mut EventPool,
    commands: &Commands,
) {
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
                link.context_menu(|ui| get_file_context_menu(ui, entry, event_pool, commands));
                if link.clicked() {
                    event_pool.emit_event(EventType::SetPath(entry.path.clone()));
                }
            }
            utils::dir::EntryKind::File(file) => {
                let resp = if file.is_exe {
                    let exe = ui.link(RichText::new(name).color(style.colors.exe));
                    if exe.clicked() {
                        event_pool.emit_event(EventType::Exec(entry.path.clone()));
                    }
                    exe
                } else {
                    ui.label(name)
                };
                resp.context_menu(|ui| {
                    get_file_context_menu(ui, entry, event_pool, commands);
                });
            }
        }
        ui.allocate_space(ui.available_size());
    })
    .context_menu(|ui| {
        get_file_context_menu(ui, entry, event_pool, commands);
    });
}

pub fn draw_file_size_cell(
    ui: &mut egui::Ui,
    entry: &FeEntry,
    event_pool: &mut EventPool,
    commands: &Commands,
) {
    cell(ui, |ui| {
        match &entry.entry_type {
            utils::dir::EntryKind::Dir(_) => {
                ui.label("");
            }
            utils::dir::EntryKind::File(file) => {
                ui.label(utils::human_readable_size(file.size).to_string())
                    .context_menu(|ui| {
                        get_file_context_menu(ui, entry, event_pool, commands);
                    });
            }
        }
        ui.allocate_space(ui.available_size());
    })
    .context_menu(|ui| {
        get_file_context_menu(ui, entry, event_pool, commands);
    });
}

pub fn draw_last_modified_cell(
    ui: &mut egui::Ui,
    entry: &FeEntry,
    event_pool: &mut EventPool,
    commands: &Commands,
) {
    cell(ui, |ui| {
        match &entry.entry_type {
            utils::dir::EntryKind::Dir(_) => {
                ui.label("");
            }
            utils::dir::EntryKind::File(file) => {
                ui.label(utils::system_time_to_human_readable(file.modified))
                    .context_menu(|ui| {
                        get_file_context_menu(ui, entry, event_pool, commands);
                    });
            }
        }
        ui.allocate_space(ui.available_size());
    })
    .context_menu(|ui| {
        get_file_context_menu(ui, entry, event_pool, commands);
    });
}

pub fn get_file_context_menu(
    ui: &mut Ui,
    entry: &FeEntry,
    event_pool: &mut EventPool,
    commands: &Commands,
) {
    let mut close = false;

    match &entry.entry_type {
        utils::dir::EntryKind::Dir(_) => {
            if ui.button("Open").clicked() {
                close = true;
                ui.close_menu();
                event_pool.emit_event(EventType::SetPath(entry.path.clone()));
            }
        }
        utils::dir::EntryKind::File(file) => {
            if file.is_exe {
                if ui.button("Run").clicked() {
                    close = true;
                    ui.close_menu();
                    event_pool.emit_event(EventType::Exec(entry.path.clone()));
                }
            }
        }
    }

    if ui.button("Properties").clicked() {
        close = true;
        ui.close_menu();
        // TODO
    }
    if ui.button("Copy path").clicked() {
        close = true;
        ui.ctx()
            .output_mut(|o| o.copied_text = entry.path.to_string_lossy().to_string());
        ui.close_menu();
    }
    if ui.button("Delete").clicked() {
        close = true;
        event_pool.emit_event(EventType::DeleteFile(entry.clone()));
    }
    if ui.button("Rename").clicked() {
        close = true;
        println!("TODO rename file");
    }

    //custom file commands
    if let utils::dir::EntryKind::File(file) = &entry.entry_type {
        if let Some(file_commands) = commands.file.as_ref() {
            for cmd in file_commands {
                if !cmd.extensions.is_none() && file.is_of_ext(cmd.extensions.clone().unwrap()) {
                    if ui.button(cmd.name.clone()).clicked() {
                        event_pool
                            .emit_event(EventType::RunFileCmd(cmd.clone(), entry.path.clone()))
                    }
                }
            }
        }
    }

    ui.separator();
    get_current_dir_context_menu(ui, event_pool, commands);

    if close {
        ui.close_menu();
    }
}

// context menu for the dir currently being browsed
pub fn get_current_dir_context_menu(ui: &mut Ui, event_pool: &mut EventPool, commands: &Commands) {
    let mut close = false;
    if ui.button("New File").clicked() {
        close = true;
        event_pool.emit_event(EventType::NewFile)
    }
    if ui.button("Open Terminal").clicked() {
        close = true;
        event_pool.emit_event(EventType::OpenTerminal)
    }

    // custom dir commands
    if let Some(dir_commands) = commands.dir.as_ref() {
        for cmd in dir_commands {
            if ui.button(cmd.name.clone()).clicked() {
                close = true;
                event_pool.emit_event(EventType::RunDirCmd(cmd.clone()));
            };
        }
    }

    if close {
        ui.close_menu();
    }
}

pub fn cell<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> Response {
    ui.horizontal(add_contents).response
}
