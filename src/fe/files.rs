use egui::Ui;
use egui_extras::{Column, TableBody, TableBuilder};
use std::{
    ffi::OsString,
    fs::{self, DirEntry},
    path::PathBuf,
};

use super::FE;

impl FE {
    // updates `path_string` and `prev_path` with current `path` content.
    // Call after updating `path`
    pub fn set_path(&mut self, path: PathBuf) {
        self.prev_path = Some(self.path.clone());
        self.path = path;
        self.path_string = self.path.to_str().unwrap().to_owned();
        self.load_dir_entries();
    }

    pub fn go_back_path(&mut self) {
        match &self.prev_path {
            Some(prev) => {
                self.set_path(prev.clone());
            }
            None => {
                println!("no previous dir to go back to")
            }
        }
    }

    // updates the internal `path` with the value in `pathString` and load the files of the new dir
    pub fn load_dir_entries(&mut self) {
        self.path = PathBuf::from(&self.path_string);
        let mut entries = Vec::new();

        match fs::read_dir(&self.path) {
            Ok(i) => {
                for entry in i {
                    let entry = entry.unwrap();
                    entries.push(entry);
                }

                self.entries = entries;
            }
            Err(err) => {
                println!("error reading entries: {:?}", err)
            }
        }
    }

    pub fn draw_files(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            let mut table = TableBuilder::new(ui)
                .striped(true)
                .resizable(false)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto())
                .column(Column::remainder())
                .min_scrolled_height(0.0)
                .max_scroll_height(600.0);

            table = table.sense(egui::Sense::click());

            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Name");
                    });
                    header.col(|ui| {
                        ui.strong("Size");
                    });
                })
                .body(|mut body| {
                    // add the '..' to go back one level
                    body.row(self.row_height, |mut row| {
                        row.col(|ui| {
                            ui.label("📁");
                            if ui.link("..").clicked() {
                                self.set_path(self.path.parent().unwrap().to_path_buf());
                                self.load_dir_entries()
                            }
                        });
                        row.col(|ui| {
                            ui.label("");
                        });
                    });

                    let mut new_path = None;
                    for entry in &self.entries {
                        new_path = draw_file_row(&mut body, entry, self.path.clone());
                    }

                    if let Some(path) = new_path {
                        self.set_path(path);
                    }
                });
        });
    }
}

pub fn draw_file_row(
    body: &mut TableBody,
    entry: &DirEntry,
    current_path: PathBuf,
) -> Option<PathBuf> {
    let mut ret = None;
    body.row(16.0, |mut row| {
        row.col(|ui| {
            ret = draw_file_name_cell(ui, &entry, &current_path);
        });
        row.col(|ui| {
            ret = draw_file_size_cell(ui, &entry, &current_path);
        });
    });

    return ret;
}

pub fn draw_file_name_cell(
    ui: &mut egui::Ui,
    entry: &DirEntry,
    current_path: &PathBuf,
) -> Option<PathBuf> {
    let mut ret = None;

    let name = entry.file_name().to_owned();
    let file_type = entry.file_type().unwrap();
    let icon = if file_type.is_dir() { "📁" } else { "📃" };

    // Create a horizontal group for the whole row
    let cell_area = ui
        .horizontal(|ui| {
            ui.label(icon);
            if file_type.is_dir() {
                let link = ui.link(&name.to_str().unwrap().to_owned());
                link.context_menu(|ui| {
                    ret = get_file_context_menu(ui, file_type, &name, &current_path);
                });
                if link.clicked() {
                    let mut new_path = current_path.clone();
                    new_path.push(&name.clone());
                    ret = Some(new_path);
                }
                // Capture the response from the whole horizontal group
            } else {
                ui.label(name.to_str().unwrap().to_owned())
                    .context_menu(|ui| {
                        ret = get_file_context_menu(ui, file_type, &name, current_path);
                    });
            }
            ui.allocate_space(ui.available_size());
        })
        .response;

    // Apply context menu to the entire row
    cell_area.context_menu(|ui| {
        ret = get_file_context_menu(ui, file_type, &name, current_path);
    });

    if cell_area.hovered() {
        // TODO: highlight row
    }

    return ret;
}

pub fn draw_file_size_cell(
    ui: &mut egui::Ui,
    entry: &DirEntry,
    current_path: &PathBuf,
) -> Option<PathBuf> {
    let mut ret = None;
    let name = entry.file_name().to_owned();
    let file_type = entry.file_type().unwrap();
    let size = entry.metadata().unwrap().len();

    let cell_area = ui
        .horizontal(|ui| {
            if file_type.is_dir() {
                ui.label("");
            } else {
                // TODO: format size
                ui.label(size.to_string()).context_menu(|ui| {
                    ret = get_file_context_menu(ui, file_type, &name, current_path);
                });
            }
            ui.allocate_space(ui.available_size());
        })
        .response;

    cell_area.context_menu(|ui| {
        ret = get_file_context_menu(ui, file_type, &name, current_path);
    });

    return ret;
}

pub fn get_file_context_menu(
    ui: &mut Ui,
    file_type: fs::FileType,
    file_name: &OsString,
    current_path: &PathBuf,
) -> Option<PathBuf> {
    let mut ret = None;
    if ui.button("Open").clicked() {
        ui.close_menu();
        // Implement open functionality
        if file_type.is_dir() {
            let mut new_path = current_path.clone();
            new_path.push(&file_name);
            ret = Some(new_path);
        }
    }
    if ui.button("Properties").clicked() {
        ui.close_menu();
        // TODO
    }

    return ret;
}
