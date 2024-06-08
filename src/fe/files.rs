use super::FE;
use super::style;
use crate::command::CommandEvent;
use crate::utils;
use crate::utils::dir::{
    fs_to_fe_entry, get_valid_new_file, DirSorting, FeEntry, QuickAccessEntry, SortOrder,
};
use egui::{Response, RichText, Ui};
use egui_extras::{Column, TableBody, TableBuilder};
use std::ffi::OsString;
use std::fs::File;
use std::{
    fs::{self},
    path::PathBuf,
};

impl FE {
    // updates `path`, `path_string` and `prev_path` with argument.
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

    pub fn delete_entry(&self, entry: FeEntry) {
        if entry.is_dir {
            fs::remove_dir_all(entry.path).unwrap_or_else(|err| {
                println!("error removing file: {:?}", err);
                // TODO: Add error to self.diagnostics here
            });
        } else {
            fs::remove_file(entry.path).unwrap_or_else(|err| {
                println!("error removing file: {:?}", err);
                // TODO: Add error to self.diagnostics here
            });
        }
    }

    // creates the file and resets the file creation state
    pub fn create_file(&mut self) {
        let is_dir = self.new_file_name.chars().last() == Some('/');
        let new_file_name = get_valid_new_file(&OsString::from(&self.new_file_name), &self.entries);

        let mut new_file_path = self.path.clone();
        new_file_path.push(new_file_name);

        let result = if is_dir {
            fs::create_dir_all(new_file_path)
        } else {
            if let Some(parent) = new_file_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }    
            File::create(&new_file_path).map(|_| ())
        };

        match result {
            Err(err) => println!("error creating file: {:?}", err),
            _ => (),
        };

        self.creating_file = false;
        self.new_file_name = "".to_owned();
        self.load_dir_entries();
    }

    // load the files of current dir
    pub fn load_dir_entries(&mut self) {
        self.path = PathBuf::from(&self.path_string);
        let mut entries = Vec::new();

        match fs::read_dir(&self.path) {
            Ok(i) => {
                for entry in i {
                    let entry: FeEntry = fs_to_fe_entry(entry.unwrap()).unwrap();
                    entries.push(entry);
                }
                self.entries = entries;

                // apply sorting
                self.update_sorting(self.dir_sorting.clone());
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
        ui.vertical(|ui| {
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
                    let mut cmd = draw_back_dir_row(&mut body, self.path.clone(), self.style.row_height, &self.style);

                    for entry in &self.entries {
                        match draw_file_row(&mut body, entry, self.style.row_height, &self.style) {
                            Some(path) => cmd = Some(path),
                            None => (),
                        }
                    }

                    if let Some(cmd) = cmd {
                        // TODO: should we emit events as we draw
                        // instead of returning an event here?
                        // (probably)
                        self.commands.emit_event(cmd);
                    }
                });
        });
    }
}

pub fn draw_back_dir_row(
    body: &mut TableBody,
    current_path: PathBuf,
    row_height: f32,
    style: &style::Style,
) -> Option<CommandEvent> {
    let entry = FeEntry {
        name: "..".into(),
        path: current_path.parent().unwrap().to_path_buf(),
        is_dir: true,
        is_exe: false,
        size: 0,
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
    });

    return ret;
}

pub fn draw_file_name_cell(ui: &mut egui::Ui, entry: &FeEntry, style: &style::Style) -> Option<CommandEvent> {
    let mut ret = None;

    let name = entry.name.to_owned().to_str().unwrap().to_owned();
    let icon = if entry.is_dir { "📁" } else { "📃" };

    cell(ui, |ui| {
        ui.label(icon);
        if entry.is_dir {
            let link = ui.link(name);
            link.context_menu(|ui| match get_file_context_menu(ui, entry) {
                Some(cmd) => ret = Some(cmd),
                None => (),
            });
            if link.clicked() {
                ret = Some(CommandEvent::SetPath(entry.path.clone()));
                println!("ret = {:?}", ret);
            }
        } else {
            let resp = if entry.is_exe {
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
        if entry.is_dir {
            ui.label("");
        } else {
            ui.label(utils::human_readable_size(entry.size).to_string())
                .context_menu(|ui| {
                    match get_file_context_menu(ui, entry) {
                        Some(cmd) => ret = Some(cmd),
                        None => (),
                    };
                });
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
    if entry.is_dir {
        if ui.button("Open").clicked() {
            ui.close_menu();
            ret = Some(CommandEvent::SetPath(entry.path.clone()));
        }
    }
    if entry.is_exe {
        if ui.button("Run").clicked() {
            ui.close_menu();
            ret = Some(CommandEvent::Run(entry.path.clone()));
        }
    }
    if ui.button("Properties").clicked() {
        ui.close_menu();
        // TODO
    }
    if ui.button("Copy path").clicked() {
        ui.ctx().output_mut(|o| o.copied_text = entry.path.to_string_lossy().to_string());
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

// checks wether the current_path is favorited
pub fn is_favorited(current_path: &PathBuf, quick_access: &Vec<QuickAccessEntry>) -> bool {
    for entry in quick_access {
        if entry.path == *current_path {
            return true;
        }
    }

    return false;
}