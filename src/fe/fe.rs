use std::path::PathBuf;

use super::FE;

impl eframe::App for FE {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // menu bar
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("Command").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);
            });
        });
        // egui::widgets::global_dark_light_mode_buttons(ui); toggle light/dark mode

        // path
        egui::TopBottomPanel::top("pathbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("back").clicked() {
                    // go back 1 level
                    match self.path.parent() {
                        Some(parent) => {
                            self.path = PathBuf::from(parent);
                            self.path_string = self.path.to_str().unwrap().to_owned();
                            self.load_dir_entries();
                        }
                        None => {} // TODO
                    }
                }

                ui.label("Path");
                // let mut dir_str = self.path.to_str().unwrap();
                let path_input = ui.text_edit_singleline(&mut self.path_string);

                // on 'enter' key press
                if path_input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    println!("lost focus due to enter");
                    // keep focus after enter
                    path_input.request_focus();
                    // change path
                    self.load_dir_entries()
                }

                if ui.button("Go").clicked() {
                    // focus path input
                    path_input.request_focus();
                    // change path
                    self.load_dir_entries()
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_files(ui);
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
                ui.separator();
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
