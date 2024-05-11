use eframe;
use std::fs;

use super::command::CommandPool;

mod fe;
mod files;

pub struct FE {
    // current dir state
    path: std::path::PathBuf,
    path_string: String,
    entries: Vec<fs::DirEntry>,
    prev_path: Option<std::path::PathBuf>,

    // search state
    _search_active: bool, // TODO implement search
    search_txt: String,

    // commands and shortcuts
    commands: CommandPool,
}

impl FE {
    /// Called once before the first frame.
    pub fn new() -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        // Default::default()
        let dir = std::env::current_dir().unwrap();
        let dir_clone = dir.clone();

        let dir_str = dir_clone.to_str().unwrap();

        let mut fe = Self {
            path: dir,
            path_string: dir_str.to_owned(),
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
        eframe::run_native("fe", native_options, Box::new(|_cc| Box::new(self)))
    }
}
