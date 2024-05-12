use egui_extras::{Column, TableBuilder};
use std::{fs, fs::DirEntry, path::PathBuf};

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
                .column(Column::remainder())
                // .column(Column::remainder())
                .min_scrolled_height(0.0)
                .max_scroll_height(600.0);

            table = table.sense(egui::Sense::click());

            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Name");
                    });
                })
                .body(|mut body| {
                    // add the '..' to go back one level
                    body.row(16.0, |mut row| {
                        row.col(|ui| {
                            ui.label("📁");
                            if ui.link("..").clicked() {
                                self.set_path(self.path.parent().unwrap().to_path_buf());
                                self.load_dir_entries()
                            }
                        });
                    });

                    let mut new_path = None;
                    for entry in &self.entries {
                        body.row(16.0, |mut row| {
                            row.col(|ui| {
                                // draw file, stores the new PathBuf returned when some dir is clicked
                                if let Some(path) = self.draw_file(ui, &entry) {
                                    new_path = Some(path);
                                }
                            });
                        });
                    }

                    if let Some(path) = new_path {
                        self.set_path(path);
                    }
                });
        });
    }

    pub fn draw_file(&self, ui: &mut egui::Ui, entry: &DirEntry) -> Option<PathBuf> {
        let mut ret = None;

        let name = entry.file_name();
        let file_type = entry.file_type().unwrap();
        let icon = if file_type.is_dir() { "📁" } else { "📃" };

        // TODO: reuse context_menu

        // Create a horizontal group for the whole row
        let response = ui
            .horizontal(|ui| {
                ui.label(icon);
                if file_type.is_dir() {
                    let link = ui.link(&name.to_str().unwrap().to_owned());
                    link.context_menu(|ui| {
                        if ui.button("Open").clicked() {
                            ui.close_menu();
                            // Implement open functionality
                            if file_type.is_dir() {
                                let mut new_path = self.path.clone();
                                new_path.push(&name);
                                ret = Some(new_path);
                            }
                        }
                        if ui.button("Properties").clicked() {
                            ui.close_menu();
                            // Implement properties functionality
                        }
                    });
                    if link.clicked() {
                        let mut new_path = self.path.clone();
                        new_path.push(&name);
                        ret = Some(new_path);
                    }
                    // Capture the response from the whole horizontal group
                    ui.allocate_space(ui.available_size()); // This ensures that the rest of the row is clickable
                } else {
                    ui.label(name.to_str().unwrap().to_owned());
                }
            })
            .response;

        // Apply context menu to the entire row
        response.context_menu(|ui| {
            if ui.button("Open").clicked() {
                ui.close_menu();
                // Implement open functionality
                if file_type.is_dir() {
                    let mut new_path = self.path.clone();
                    new_path.push(name);
                    ret = Some(new_path);
                }
            }
            if ui.button("Properties").clicked() {
                ui.close_menu();
                // TODO
            }
        });

        return ret;
    }
}
