use std::path::PathBuf;

use crate::commands::Commands;
use crate::events::{EventPool, EventType};
use crate::fe::style;
use crate::utils;
use crate::utils::dir::{get_parent, FeEntry};
use egui::{Response, RichText, Ui};
use egui_extras::TableBody;

pub fn draw_back_dir_row(
    body: &mut TableBody,
    current_path: PathBuf,
    style: &style::Style,
    event_pool: &mut EventPool,
    commands: &Commands,
) -> bool {
    if let Some(entry) = get_parent(current_path) {
        return draw_file_row(body, &entry, style, event_pool, commands);
    }
    return false;
}

pub fn draw_file_row(
    body: &mut TableBody,
    entry: &FeEntry,
    style: &style::Style,
    event_pool: &mut EventPool,
    commands: &Commands,
) -> bool {
    let mut responses = Vec::new();
    let mut hovered = false;

    body.row(style.row_height, |mut row| {
        responses.push(row.col(|ui| draw_file_name_cell(ui, entry, style, event_pool, commands)));
        responses.push(row.col(|ui| draw_file_size_cell(ui, entry, event_pool, commands)));
        responses.push(row.col(|ui| draw_last_modified_cell(ui, entry, event_pool, commands)));
        if row.response().hovered() {
            hovered = true;
        }
    });

    // if at this point the row isn't hovered, check if any cell is hovered
    if !hovered {
        let hover_pos = responses
            .get(0)
            .and_then(|(_, response)| response.ctx.input(|i| i.pointer.hover_pos()));

        hovered = if let Some(hover_pos) = hover_pos {
            responses.iter().any(|(rect, _)| rect.contains(hover_pos))
        } else {
            false
        };
    }

    for (_, response) in &responses {
        response.context_menu(|ui| get_file_context_menu(ui, entry, event_pool, commands));
        if response.drag_started() {
            event_pool.emit_event(EventType::StartDragEntry(entry.clone()));
        }
    }

    return hovered;
}

pub fn draw_file_name_cell(
    ui: &mut egui::Ui,
    entry: &FeEntry,
    style: &style::Style,
    event_pool: &mut EventPool,
    commands: &Commands,
) {
    let name = entry.name.to_owned().to_str().unwrap().to_owned();
    let icon = entry.get_icon();

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
                let resp = if let Some(e) = file.is_clickable(&commands.file) {
                    let link = ui.link(RichText::new(name).color(style.colors.exe));
                    if link.clicked() {
                        if file.is_exe {
                            event_pool.emit_event(EventType::Exec(entry.path.clone()));
                        }
                        event_pool.emit_event(e);
                    }
                    link
                } else {
                    ui.label(name)
                };
                resp.context_menu(|ui| {
                    get_file_context_menu(ui, entry, event_pool, commands);
                });
            }
        }
    });
}

pub fn draw_file_size_cell(
    ui: &mut egui::Ui,
    entry: &FeEntry,
    event_pool: &mut EventPool,
    commands: &Commands,
) {
    cell(ui, |ui| match &entry.entry_type {
        utils::dir::EntryKind::Dir(_) => {
            ui.label("");
        }
        utils::dir::EntryKind::File(file) => {
            ui.label(utils::human_readable_size(file.size).to_string())
                .context_menu(|ui| {
                    get_file_context_menu(ui, entry, event_pool, commands);
                });
        }
    });
}

pub fn draw_last_modified_cell(
    ui: &mut egui::Ui,
    entry: &FeEntry,
    event_pool: &mut EventPool,
    commands: &Commands,
) {
    cell(ui, |ui| match &entry.entry_type {
        utils::dir::EntryKind::Dir(_) => {
            ui.label("");
        }
        utils::dir::EntryKind::File(file) => {
            ui.label(utils::system_time_to_human_readable(file.modified))
                .context_menu(|ui| {
                    get_file_context_menu(ui, entry, event_pool, commands);
                });
        }
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
