use diagnostic::Diagnostic;
use directories::UserDirs;
use eframe;
use egui::{Align2, Response, Sense, Ui, Vec2};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use crate::command::{CommandEvent, CommandPool};
use crate::storage;
use crate::utils::dir::{DirSorting, FeEntry, QuickAccessEntry, SortOrder};
use crate::utils::{self, term};

use self::draw::get_current_dir_context_menu;
mod diagnostic;
mod draw;
mod files;
mod style;

pub struct FE {
    // current dir state
    path: std::path::PathBuf,
    path_string: String,
    entries: Vec<FeEntry>,
    prev_path: Option<std::path::PathBuf>,
    dir_sorting: DirSorting,

    // data storage
    storage: storage::Storage,
    quick_access: Vec<QuickAccessEntry>,

    // search state
    // TODO use an Option<String> as search instead of String + bool
    _search_active: bool, // TODO implement search
    search_txt: String,

    // commands and shortcuts
    commands: CommandPool,

    // styles
    style: style::Style,

    creating_file: bool,
    new_file_name: String,

    // error diagnostics
    diagnostics: Vec<Diagnostic>,
}

impl FE {
    pub fn new() -> Self {
        let dir = std::env::current_dir().unwrap();
        let dir_clone = dir.clone();

        // TODO read dir from CLI/config file

        let mut data_path = home::home_dir().unwrap();
        data_path.push(".fe/data");
        println!("data_path: {:?}", data_path);
        fs::create_dir_all(data_path.parent().unwrap()).expect("cant create data dir");
        let storage = storage::Storage::new(data_path).unwrap();

        let quick_access_entries = storage.list_quick_access();

        let mut fe = Self {
            path: dir,
            path_string: dir_clone.to_str().unwrap().to_owned(),
            entries: Vec::new(),
            prev_path: None,
            dir_sorting: DirSorting::FileNameAlphabetically(SortOrder::Asc),
            storage: storage,
            quick_access: quick_access_entries,
            _search_active: false,
            search_txt: "".to_owned(),
            commands: CommandPool::new(),
            style: style::Style::default(),
            creating_file: false,
            new_file_name: "".to_owned(),
            diagnostics: Vec::new(),
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

    fn handle_events(&mut self, _ctx: &egui::Context) -> Option<()> {
        let events: Vec<CommandEvent> = self.commands.get_events();

        for event in events {
            match event {
                CommandEvent::DirGoBack => {
                    self.go_back_path();
                }
                CommandEvent::FavoriteCurrentPath => {
                    if !files::is_favorited(&self.path, &self.quick_access) {
                        let entry = QuickAccessEntry {
                            name: self.path.file_name()?.to_os_string(),
                            path: self.path.clone(),
                        };
                        // update storage
                        self.storage.save_quick_access(entry.clone());
                        // update memory
                        self.quick_access.push(entry);
                    } else {
                        self.storage.remove_quick_access(&self.path);
                        self.quick_access.retain(|entry| entry.path != self.path);
                    }
                }
                CommandEvent::NewFile => {
                    self.creating_file = true;
                }
                CommandEvent::SetPath(path) => {
                    self.set_path(path.clone());
                }
                CommandEvent::OpenTerminal => {
                    term::open_terminal(self.path_string.as_str());
                }
                CommandEvent::DeleteFile(entry) => {
                    self.delete_entry(entry);
                    self.load_dir_entries();
                }
                CommandEvent::Run(path) => {
                    if let Err(err) = utils::run_exe(&path) {
                        self.diagnostics.push(Diagnostic::from_err(&err));
                    };
                }
                _ => {}
            }
        }
        return Some(());
    }

    fn draw_diagnostics(&mut self, ctx: &egui::Context) {
        self.diagnostics.retain(|d| d.expires_at > Instant::now());

        if self.diagnostics.is_empty() {
            return;
        }

        let window = egui::Window::new("Diagnostics")
            .title_bar(true)
            .resizable(false)
            .constrain(true)
            .collapsible(false);

        let align = Align2([egui::Align::RIGHT, egui::Align::BOTTOM]);
        let offset = Vec2::new(-16.0, -16.0);
        window.anchor(align, offset).show(ctx, |ui| {
            for diagnostic in &self.diagnostics {
                ui.label(&diagnostic.message);
                // TODO: draw a progress bar maybe
            }
        });
    }
}

impl eframe::App for FE {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.commands.emit_input_events(ctx);
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
                            None => {
                                self.diagnostics
                                    .push(Diagnostic::default("no parent".to_string()));
                            }
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

                    let favorited = files::is_favorited(&self.path, &self.quick_access);
                    if ui.button(if favorited { "🌟" } else { "⭐" }).clicked() {
                        self.commands.emit_event(CommandEvent::FavoriteCurrentPath);
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
            // left part, pinned dirs
            egui::SidePanel::left("left_panel")
                .resizable(false)
                .default_width(150.0)
                .show_inside(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label("quick access");
                        ui.separator();
                        // TODO: Home and Desktop should be stored to quick links on first load
                        // then, read from storage instead of hardcoding it here
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

                        let quick_access = self.quick_access.clone();
                        for entry in quick_access {
                            if ui
                                .link(format!(
                                    "{} {}",
                                    "📁",
                                    entry.name.to_str().unwrap().to_owned(),
                                ))
                                .clicked()
                            {
                                self.set_path(entry.path);
                            }
                        }
                    });
                });

            // right part, file list
            egui::CentralPanel::default().show_inside(ui, |ui| {
                self.draw_files(ui);
                // Create an invisible panel to handle the right-click
                fill_remainder(ui).context_menu(|ui| {
                    get_current_dir_context_menu(ui, &mut self.commands);
                });
            });
        });

        if self.creating_file {
            let window = egui::Window::new("new_file_window")
                .title_bar(false)
                .resizable(false)
                .constrain(true)
                .collapsible(false);

            window.show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;
                    ui.label("New file:");
                    ui.spacing_mut().item_spacing.y = 8.0;

                    let new_file_input = ui.text_edit_singleline(&mut self.new_file_name);
                    new_file_input.request_focus();

                    if new_file_input.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.create_file();
                    }

                    ui.horizontal(|ui| {
                        if ui.button("Ok").clicked() {
                            self.create_file();
                        }
                        if ui.button("Cancel").clicked() {
                            self.creating_file = false;
                        }
                    });
                })
            });
        }

        self.draw_diagnostics(ctx);

        // handle events after drawing everything, since
        // the drawing methods may emit events
        self.handle_events(ctx);
    }
}

// fills the remainder of the panel with an invisible rect capable
// of detecting clicks.
fn fill_remainder(ui: &mut Ui) -> Response {
    let rect = ui.available_rect_before_wrap();
    return ui.allocate_rect(rect, Sense::click());
}
