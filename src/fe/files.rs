use crate::utils::dir::{DirSorting, SortOrder};
use egui::{Response, Ui};
use egui_extras::{Column, TableBody, TableBuilder};
use std::{
    ffi::OsString,
    fs::{self, DirEntry},
    path::PathBuf,
};

use crate::utils;

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

                entries.sort_by(|a, b| utils::dir::compare_entries(a, b, &self.dir_sorting));
                self.entries = entries;
            }
            Err(err) => {
                println!("error reading entries: {:?}", err)
            }
        }
    }

    // update the sorting without reloading files from the file system
    pub fn update_sorting(&mut self, sort: DirSorting) {
        self.dir_sorting = sort;
        self.entries
            .sort_by(|a, b| utils::dir::compare_entries(a, b, &self.dir_sorting));
    }

    // drawing
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

                        if ui.button(self.dir_sorting.get_sort_icon()).clicked() {
                            // if currently sorting by name, toggle it
                            // otherwise, sort by name down
                            match &self.dir_sorting {
                                DirSorting::FileNameAlphabetically(dir) => {
                                    self.update_sorting(DirSorting::FileNameAlphabetically(
                                        dir.toggle(),
                                    ));
                                }
                                _ => {
                                    self.update_sorting(DirSorting::FileNameAlphabetically(
                                        SortOrder::Asc,
                                    ));
                                }
                            }
                        }
                    });
                    header.col(|ui| {
                        ui.strong("Size");
                        if ui.button(self.dir_sorting.get_sort_icon()).clicked() {
                            match &self.dir_sorting {
                                DirSorting::FileSize(dir) => {
                                    self.update_sorting(DirSorting::FileSize(dir.toggle()))
                                }
                                _ => self.update_sorting(DirSorting::FileSize(SortOrder::Asc)),
                            }
                        }
                    });
                })
                .body(|mut body| {
                    let mut new_path =
                        draw_back_dir_row(&mut body, self.path.clone(), self.row_height);

                    for entry in &self.entries {
                        match draw_file_row(&mut body, entry, self.path.clone(), self.row_height) {
                            Some(path) => new_path = Some(path),
                            None => (),
                        }
                    }

                    if let Some(path) = new_path {
                        self.set_path(path);
                    }
                });
        });
    }
}

pub fn draw_back_dir_row(
    body: &mut TableBody,
    current_path: PathBuf,
    row_height: f32,
) -> Option<PathBuf> {
    let mut ret = None;

    body.row(row_height, |mut row| {
        row.col(|ui| {
            cell(ui, |ui| {
                ui.label("📁");
                if ui.link("..").clicked() {
                    ret = Some(current_path.parent().unwrap().to_path_buf());
                }
            })
            .context_menu(|ui| {
                get_file_context_menu(ui, true, &OsString::from("..".to_string()), &current_path);
            });
        });
        row.col(|ui| {
            cell(ui, |ui| {
                ui.label("");
            });
        });
    });
    return ret;
}

pub fn draw_file_row(
    body: &mut TableBody,
    entry: &DirEntry,
    current_path: PathBuf,
    row_height: f32,
) -> Option<PathBuf> {
    let mut ret = None;
    body.row(row_height, |mut row| {
        row.col(|ui| match draw_file_name_cell(ui, &entry, &current_path) {
            Some(path) => ret = Some(path),
            None => (),
        });
        row.col(|ui| match draw_file_size_cell(ui, &entry, &current_path) {
            Some(path) => ret = Some(path),
            None => (),
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

    cell(ui, |ui| {
        ui.label(icon);
        if file_type.is_dir() {
            let link = ui.link(&name.to_str().unwrap().to_owned());
            link.context_menu(|ui| {
                match get_file_context_menu(ui, file_type.is_dir(), &name, &current_path) {
                    Some(path) => ret = Some(path),
                    None => (),
                }
            });
            if link.clicked() {
                let mut new_path = current_path.clone();
                new_path.push(&name.clone());
                ret = Some(new_path);
                println!("ret = {:?}", ret);
            }
        } else {
            ui.label(name.to_str().unwrap().to_owned())
                .context_menu(|ui| {
                    match get_file_context_menu(ui, file_type.is_dir(), &name, current_path) {
                        Some(path) => ret = Some(path),
                        None => (),
                    };
                });
        }
        ui.allocate_space(ui.available_size());
    })
    .context_menu(|ui| {
        match get_file_context_menu(ui, file_type.is_dir(), &name, current_path) {
            Some(path) => ret = Some(path),
            None => (),
        };
    });

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

    cell(ui, |ui| {
        if file_type.is_dir() {
            ui.label("");
        } else {
            // TODO: format size
            ui.label(utils::human_readable_size(size).to_string())
                .context_menu(|ui| {
                    match get_file_context_menu(ui, file_type.is_dir(), &name, current_path) {
                        Some(path) => ret = Some(path),
                        None => (),
                    };
                });
        }
        ui.allocate_space(ui.available_size());
    })
    .context_menu(|ui| {
        match get_file_context_menu(ui, file_type.is_dir(), &name, current_path) {
            Some(path) => ret = Some(path),
            None => (),
        };
    });

    return ret;
}

pub fn get_file_context_menu(
    ui: &mut Ui,
    is_dir: bool,
    file_name: &OsString,
    current_path: &PathBuf,
) -> Option<PathBuf> {
    // TODO: have a dir-wide context menu items
    // and concatanate it file menu items to
    // create the final context menu.
    // When clicking in the table but not on any file
    // we can show the dir menu only.
    let mut ret = None;
    if is_dir {
        if ui.button("Open").clicked() {
            ui.close_menu();
            // Implement open functionality
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

pub fn cell<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> Response {
    ui.horizontal(add_contents).response
}
