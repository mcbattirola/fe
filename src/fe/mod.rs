use eframe;
use std::fs;

mod fe;
mod files;

pub struct FE {
    // Example stuff:
    path: String,
    entries: Vec<fs::DirEntry>,
}

impl FE {
    /// Called once before the first frame.
    pub fn new() -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        // Default::default()
        let dir = std::env::current_dir().unwrap();
        let dir_str = dir.to_str().unwrap();

        let mut fe = Self {
            path: dir_str.to_owned(),
            entries: Vec::new(),
        };

        fe.load_dir_entries(dir_str.to_string());

        return fe;
    }

    pub fn run(self) -> Result<(), eframe::Error> {
        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([400.0, 300.0])
                .with_min_inner_size([300.0, 220.0]),
            ..Default::default()
        };
        eframe::run_native("fe", native_options, Box::new(|cc| Box::new(self)))
    }

    fn load_dir_entries(&mut self, dir: String) {
        let mut entries = Vec::new();

        match fs::read_dir(dir) {
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
}
