use super::draw;
use super::FE;
use crate::fe::diagnostic::Diagnostic;
use crate::utils;
use crate::utils::dir::get_sort_icon;
use crate::utils::dir::DirSortingType;
use crate::utils::dir::{
    fs_to_fe_entry, get_valid_new_file, DirSorting, FeEntry, QuickAccessEntry, SortOrder,
};
use egui_extras::{Column, TableBuilder};
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
        self.search_txt = String::new();
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

    pub fn delete_entry(&mut self, entry: FeEntry) {
        match entry.entry_type {
            utils::dir::EntryKind::Dir(_) => {
                if let Err(err) = fs::remove_dir_all(entry.path) {
                    self.diagnostics.push(Diagnostic::from_err(&err));
                }
            }
            utils::dir::EntryKind::File(_) => {
                if let Err(err) = fs::remove_file(entry.path) {
                    self.diagnostics.push(Diagnostic::from_err(&err));
                }
            }
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

        if let Err(err) = result {
            self.diagnostics.push(Diagnostic::from_err(&err));
        };

        self.creating_file = false;
        self.new_file_name = "".to_owned();
        self.load_dir_entries();
    }

    pub fn move_file(&mut self, origin: &PathBuf, dest: &PathBuf) {
        if let Err(err) = fs::rename(origin, dest) {
            self.diagnostics.push(Diagnostic::from_err(&err))
        }
    }

    // load the files of current dir. Prefer calling set_path if updating the path,
    // use this only for reloading the current directory's files.
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
                self.diagnostics.push(Diagnostic::from_err(&err));
                println!("error reading entries: {:?}", err)
            }
        }
        self.update_display_entries();
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
                .column(Column::auto())
                .column(Column::remainder())
                .min_scrolled_height(0.0)
                .max_scroll_height(600.0);

            table = table.sense(egui::Sense::click());

            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Name");

                        if ui
                            .button(get_sort_icon(
                                DirSortingType::FileNameAlphabetically,
                                &self.dir_sorting,
                            ))
                            .clicked()
                        {
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
                        if ui
                            .button(get_sort_icon(DirSortingType::FileSize, &self.dir_sorting))
                            .clicked()
                        {
                            match &self.dir_sorting {
                                DirSorting::FileSize(dir) => {
                                    self.update_sorting(DirSorting::FileSize(dir.toggle()))
                                }
                                _ => self.update_sorting(DirSorting::FileSize(SortOrder::Asc)),
                            }
                        }
                    });
                    header.col(|ui| {
                        ui.strong("Modified");
                        if ui
                            .button(get_sort_icon(
                                DirSortingType::LastModified,
                                &self.dir_sorting,
                            ))
                            .clicked()
                        {
                            match &self.dir_sorting {
                                DirSorting::LastModified(dir) => {
                                    self.update_sorting(DirSorting::LastModified(dir.toggle()))
                                }
                                _ => self.update_sorting(DirSorting::LastModified(SortOrder::Asc)),
                            }
                        }
                    });
                })
                .body(|mut body| {
                    let _cmd = draw::file::draw_back_dir_row(
                        &mut body,
                        self.path.clone(),
                        &self.style,
                        &mut self.event_pool,
                        &self.commands,
                    );

                    for entry in &self.display_entries {
                        draw::file::draw_file_row(
                            &mut body,
                            entry,
                            &self.style,
                            &mut self.event_pool,
                            &self.commands,
                        );
                    }
                });
        });
    }
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
