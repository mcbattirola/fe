use std::fs::DirEntry;

use super::FE;

pub fn draw_files(ui: &mut egui::Ui, fe: &FE) {
        // The central panel the region left after adding TopPanel's and SidePanel's
        ui.heading("Files");
        ui.vertical_centered_justified(|ui| {
            for entry in &fe.entries {
                ui.horizontal(|ui| {
                    draw_file(ui, entry);
                });
            }
        });
}

pub fn draw_file(ui: &mut egui::Ui, entry: &DirEntry) {
    let name = entry.file_name();
    ui.label(name.to_str().unwrap().to_owned());
}
