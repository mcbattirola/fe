use diagnostic::Diagnostic;
use directories::UserDirs;
use eframe::{self};
use egui::{Align2, Response, Sense, Ui, Vec2};
use std::fs;
use std::path::{PathBuf, MAIN_SEPARATOR};
use std::time::Instant;

use crate::commands::Commands;
use crate::config::{parse_config, Config};
use crate::events::{EventPool, EventType};
use crate::utils::dir::{DirSorting, EntryKind, FeEntry, QuickAccessEntry, SortOrder};
use crate::utils::{self, term};
use crate::{cli, commands, storage};

use self::draw::file::get_current_dir_context_menu;
mod diagnostic;
mod draw;
mod files;
mod style;

pub struct FE {
    // current dir state
    path: std::path::PathBuf,
    path_string: String,
    entries: Vec<FeEntry>,
    display_entries: Vec<FeEntry>,

    prev_path: Option<std::path::PathBuf>,
    dir_sorting: DirSorting,

    // data storage
    storage: storage::Storage,
    quick_access: Vec<QuickAccessEntry>,

    // search state
    search_txt: String,

    // ui events and shortcuts
    event_pool: EventPool,

    // styles
    style: style::Style,

    creating_file: bool,
    new_file_name: String,

    // error diagnostics
    diagnostics: Vec<Diagnostic>,

    // custom commands
    commands: commands::Commands,
    hovered_file: Option<FeEntry>,

    dragging_file: Option<FeEntry>,
}

impl FE {
    // creates FE from Config (usually parsed from a config file)
    pub fn from_config(config: Config) -> Self {
        let path = std::env::current_dir().unwrap();
        let path_string = path.clone().to_str().unwrap().to_owned();

        let data_path: PathBuf = config.data_dir.expect("data_path is empty").into();
        println!("data_path: {:?}", data_path);
        fs::create_dir_all(data_path.parent().unwrap()).expect("cant create data dir");
        let storage = storage::Storage::new(data_path).unwrap();

        let quick_access_entries = storage.list_quick_access().unwrap();

        let commands = match config.commands {
            None => Commands {
                file: None,
                dir: None,
            },
            Some(c) => c,
        };

        let mut fe = Self {
            path,
            path_string,
            entries: Vec::new(),
            display_entries: Vec::new(),
            prev_path: None,
            dir_sorting: DirSorting::FileNameAlphabetically(SortOrder::Asc),
            storage: storage,
            quick_access: quick_access_entries,
            search_txt: "".to_owned(),
            event_pool: EventPool::new(),
            style: style::Style::default(),
            creating_file: false,
            new_file_name: "".to_owned(),
            diagnostics: Vec::new(),
            commands,
            hovered_file: None,
            dragging_file: None,
        };

        fe.load_dir_entries();

        return fe;
    }

    // creates FE from CliArgs
    pub fn from_args(args: cli::CliArgs) -> Self {
        let config_path = args.config_path.unwrap();
        let config = parse_config(&config_path);
        if config.is_err() {
            panic!(
                "cannot parse config file {:?}: {:?}",
                config_path,
                config.err()
            );
        }
        return FE::from_config(config.unwrap());
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

    fn update_display_entries(&mut self) {
        self.display_entries = match self.search_txt.as_str() {
            "" => self.entries.clone(),
            _ => self
                .entries
                .iter()
                .filter(|e| e.name.to_string_lossy().contains(&self.search_txt))
                .cloned()
                .collect(),
        };
    }

    fn handle_events(&mut self) -> Option<()> {
        for event in self.event_pool.get_events() {
            match event {
                EventType::DirGoBack => {
                    self.go_back_path();
                }
                EventType::FavoriteCurrentPath => {
                    if !files::is_favorited(&self.path, &self.quick_access) {
                        let entry = QuickAccessEntry {
                            name: self.path.file_name()?.to_os_string(),
                            path: self.path.clone(),
                        };
                        // update storage
                        if let Err(err) = self.storage.save_quick_access(&entry) {
                            self.diagnostics.push(Diagnostic::from_err(&err));
                        }
                        // update memory
                        self.quick_access.push(entry);
                    } else {
                        if let Err(err) = self.storage.remove_quick_access(&self.path) {
                            self.diagnostics.push(Diagnostic::from_err(&err));
                        }
                        self.quick_access.retain(|entry| entry.path != self.path);
                    }
                }
                EventType::NewFile => {
                    self.creating_file = true;
                }
                EventType::SetPath(path) => {
                    self.set_path(path.clone());
                }
                EventType::OpenTerminal => {
                    if let Some(err) = term::open_terminal(self.path_string.as_str()) {
                        self.diagnostics.push(Diagnostic::from_err(&err));
                    };
                }
                EventType::DeleteFile(entry) => {
                    self.delete_entry(entry);
                    self.load_dir_entries();
                }
                EventType::Exec(path) => {
                    if let Err(err) = utils::run_exe(&path) {
                        self.diagnostics.push(Diagnostic::from_err(&err));
                    };
                }
                EventType::RunFileCmd(cmd, file_path) => {
                    if let Err(err) = cmd.run(&file_path) {
                        self.diagnostics.push(Diagnostic::from_err(&err.as_ref()));
                    };
                }
                EventType::RunDirCmd(cmd) => {
                    if let Err(err) = cmd.run(&self.path) {
                        self.diagnostics.push(Diagnostic::from_err(&err.as_ref()));
                    };
                }
                EventType::ReloadDir => {
                    self.set_path(self.path.clone());
                }
                EventType::MoveFile(3, files) => {
                    for file in files.iter() {
                        if let Some(path) = &file.path {
                            let file_name = path.file_name().unwrap_or_default();
                            // drop either in the current dir or the hovered dir
                            let dest_path = match self.hovered_file {
                                None => self.path.join(file_name),
                                Some(ref entry) => match entry.entry_type {
                                    EntryKind::Dir(_) => entry.path.join(file_name),
                                    EntryKind::File(_) => self.path.join(file_name),
                                },
                            };
                            self.move_file(path, &dest_path);
                            self.load_dir_entries();
                        }
                    }
                }
                // we emit again the MoveFile because dragging and dropping a file
                // from another app into fe will cause the mouse to only be detected
                // in the next frame. This way, we can capture the area in which the
                // file was dropped.
                EventType::MoveFile(num, files) => {
                    self.event_pool
                        .schedule_event(EventType::MoveFile(num + 1, files));
                }
                EventType::StartDragEntry(entry) => {
                    self.dragging_file = Some(entry);
                }
                EventType::EndDragEntry => {
                    if let Some(file) = &self.dragging_file {
                        if let Some(dest) = &self.hovered_file {
                            match dest.entry_type {
                                EntryKind::Dir(_) => {
                                    // Extract paths to avoid multiple mutable borrows
                                    let file_path = file.path.clone();
                                    let mut dest_path = dest.path.clone();
                                    dest_path.push(file.name.clone());
                                    self.move_file(&file_path, &dest_path);
                                    self.load_dir_entries();
                                }
                                EntryKind::File(_) => (),
                            }
                        }
                    }
                    self.dragging_file = None;
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
        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
            self.event_pool
                .emit_event(EventType::MoveFile(0, dropped_files));
        }

        self.event_pool.emit_input_events(ctx);
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
                        self.event_pool
                            .emit_event(EventType::SetPath(self.path_string.clone().into()));
                        // append a '/' at the end of the path string
                        if !self.path_string.ends_with(MAIN_SEPARATOR) {
                            self.path_string.push(MAIN_SEPARATOR);

                            // move cursor to last position
                            if let Some(mut state) =
                                egui::TextEdit::load_state(ui.ctx(), path_input.id)
                            {
                                let ccursor =
                                    egui::text::CCursor::new(self.path_string.chars().count());
                                state
                                    .cursor
                                    .set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                                state.store(ui.ctx(), path_input.id);
                                ui.ctx().memory_mut(|mem| mem.request_focus(path_input.id));
                            }
                        }
                    }

                    if self.event_pool.get_event(EventType::FocusPathBar) {
                        path_input.request_focus();
                    }

                    if ui.button("Go").clicked() {
                        // focus path input
                        path_input.request_focus();
                        self.event_pool
                            .emit_event(EventType::SetPath(self.path_string.clone().into()));
                    }

                    let favorited = files::is_favorited(&self.path, &self.quick_access);
                    if ui.button(if favorited { "🌟" } else { "⭐" }).clicked() {
                        self.event_pool.emit_event(EventType::FavoriteCurrentPath);
                    }
                });

                // search bar
                ui.horizontal(|ui| {
                    ui.separator();
                    ui.label("Seach");
                    let search_input = ui.text_edit_singleline(&mut self.search_txt);
                    if self.event_pool.get_event(EventType::FocusSearchBar) {
                        search_input.request_focus();
                    }
                    if search_input.changed() {
                        self.update_display_entries();
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
                                match user_dirs.desktop_dir() {
                                    Some(dir) => self.set_path(dir.to_path_buf()),
                                    None => println!("no desktop dir found"),
                                }
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
                    get_current_dir_context_menu(ui, &mut self.event_pool, &self.commands);
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

        if ctx.input(|i| i.pointer.any_released()) {
            self.event_pool.emit_event(EventType::EndDragEntry);
        }

        // draw dragging file
        // TODO:
        // - move this to a method,
        // - improve style
        // - hover effect over hovered dirs
        egui::Area::new(egui::Id::new("dragging_label_area"))
            .fixed_pos(egui::pos2(0.0, 0.0)) // This position can be adjusted as needed
            .show(ctx, |ui| {
                if let Some(dragging) = &self.dragging_file {
                    // Get the current mouse position
                    if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                        let draw_pos = egui::Pos2::new(pos.x + 5.0, pos.y - 5.0);
                        // Set the position for the label and draw it
                        ui.painter().text(
                            draw_pos,
                            egui::Align2::LEFT_TOP,
                            format!(
                                "{} {}",
                                dragging.get_icon(),
                                dragging.name.clone().into_string().unwrap()
                            ),
                            egui::FontId {
                                size: 16.0,
                                family: ctx.style().text_styles[&egui::TextStyle::Body]
                                    .family
                                    .clone(),
                            },
                            ui.visuals().text_color(),
                        );
                    }
                }
            });

        // handle events after drawing everything, since
        // the drawing methods may emit events
        self.handle_events();
        self.event_pool.flush_events();
    }
}

// fills the remainder of the panel with an invisible rect capable
// of detecting clicks.
fn fill_remainder(ui: &mut Ui) -> Response {
    let rect = ui.available_rect_before_wrap();
    return ui.allocate_rect(rect, Sense::click());
}
