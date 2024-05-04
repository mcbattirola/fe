use std::fs::DirEntry;

use super::super::FE;

impl FE {
    pub fn draw_files(&self, ui: &mut egui::Ui) {
        // The central panel the region left after adding TopPanel's and SidePanel's
        ui.heading("Files");
        ui.vertical_centered_justified(|ui| {
            for entry in &self.entries {
                ui.horizontal(|ui| {
                    self.draw_file(ui, entry);
                });
            }
        });
    }

    pub fn draw_file(&self, ui: &mut egui::Ui, entry: &DirEntry) {
        let name = entry.file_name();
        ui.label(name.to_str().unwrap().to_owned());
    }
}
