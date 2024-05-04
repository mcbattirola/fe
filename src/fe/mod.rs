use eframe;
use std::fs;

mod fe;
mod render;

pub struct FE {
    // Example stuff:
    path: std::path::PathBuf,
    path_string: String,
    entries: Vec<fs::DirEntry>,
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
        };

        fe.load_dir_entries();

        return fe;
    }

    pub fn run(self) -> Result<(), eframe::Error> {
        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([400.0, 300.0])
                .with_min_inner_size([300.0, 220.0])
                .with_resizable(true),
            ..Default::default()
        };
        eframe::run_native("fe", native_options, Box::new(|_cc| Box::new(self)))
    }
}
