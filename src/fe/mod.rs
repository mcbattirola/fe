use directories::UserDirs;
use eframe;
use std::fs;
use std::path::PathBuf;

use crate::command::{CommandEvent, CommandPool};

mod files;

pub struct FE {
    // current dir state
    path: std::path::PathBuf,
    path_string: String,
    entries: Vec<fs::DirEntry>,
    prev_path: Option<std::path::PathBuf>,

    // search state
    // TODO use an Option<String> as search instead of String + bool
    _search_active: bool, // TODO implement search
    search_txt: String,

    // commands and shortcuts
    commands: CommandPool,
}

impl FE {
    pub fn new() -> Self {
        let dir = std::env::current_dir().unwrap();
        let dir_clone = dir.clone();

        let mut fe = Self {
            path: dir,
            path_string: dir_clone.to_str().unwrap().to_owned(),
            entries: Vec::new(),
            prev_path: None,
            _search_active: false,
            search_txt: "".to_owned(),
            commands: CommandPool::new(),
        };

        fe.load_dir_entries();

        return fe;
    }

    pub fn run(self) -> Result<(), eframe::Error> {
        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([800.0, 600.0])
                .with_min_inner_size([800.0, 600.0])
                .with_resizable(true),
            ..Default::default()
        };
        eframe::run_native("fe", native_options, Box::new(|cc| Box::new(self.init(cc))))
    }

    // init runs the initial style setup
    fn init(self, cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_pixels_per_point(1.2);
        self
    }
}

impl eframe::App for FE {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.commands.update(ctx);

        if self.commands.get_event(CommandEvent::DirGoBack) {
            self.go_back_path();
        }

        if self.commands.get_event(CommandEvent::FavoritePath) {
            println!("TODO: favorite current path");
        }

        // menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("Command").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });

        // path and search bars
        egui::TopBottomPanel::top("top-bars").show(ctx, |ui| {
            // path
            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("↩").clicked() {
                        self.go_back_path();
                    }
                    if ui.button("⬆").clicked() {
                        // go back 1 level
                        match self.path.parent() {
                            Some(parent) => {
                                self.set_path(PathBuf::from(parent));
                            }
                            None => {} // TODO
                        }
                    }

                    ui.label("Path");
                    let path_input = ui.text_edit_singleline(&mut self.path_string);

                    // on 'enter' key press
                    if path_input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        // keep focus after enter
                        path_input.request_focus();
                        // change path
                        self.load_dir_entries()
                    }

                    if self.commands.get_event(CommandEvent::FocusPathBar) {
                        path_input.request_focus();
                    }

                    if ui.button("Go").clicked() {
                        // focus path input
                        path_input.request_focus();
                        // change path
                        self.load_dir_entries()
                    }

                    if ui.button("⭐").clicked() {
                        println!("TODO: add {:?} to favorites", self.path_string);
                    }
                });

                // search bar
                ui.horizontal(|ui| {
                    ui.separator();
                    ui.label("Seach");
                    let search_input = ui.text_edit_singleline(&mut self.search_txt);
                    if self.commands.get_event(CommandEvent::FocusSearchBar) {
                        search_input.request_focus();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // left part
            egui::SidePanel::left("left_panel")
                .resizable(false)
                .default_width(150.0)
                .show_inside(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label("quick access");
                        ui.separator();
                        // TODO organize this part
                        if ui.link("🏠 Home").clicked() {
                            if home::home_dir().is_some() {
                                self.set_path(home::home_dir().unwrap());
                            }
                        };
                        if ui.link("📺 Desktop").clicked() {
                            if let Some(user_dirs) = UserDirs::new() {
                                self.set_path(user_dirs.desktop_dir().unwrap().to_path_buf());
                            }
                        }
                    });
                });

            // right part
            egui::CentralPanel::default().show_inside(ui, |ui| {
                self.draw_files(ui);
            });

            // ui.add_space(30.0);
        });
    }
}
