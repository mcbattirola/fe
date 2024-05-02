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
        let _path_resp = egui::TopBottomPanel::top("pathbar")
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Path");
                    let path_input = ui.text_edit_singleline(&mut self.path);
                    if path_input.changed() {
                        println!("path: {:?}", self.path)
                    }
                    // on 'enter' key press
                    if path_input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        println!("lost focus due to enter");
                        // keep focus after enter
                        path_input.request_focus();
                        self.load_dir_entries(self.path.to_string())
                        // change path
                    }
                });
            })
            .response;

        // hover example:
        // print!("hovered? {:?}\n", _path_resp.hovered());

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Files");
            ui.vertical_centered_justified(|ui| {
                for entry in &self.entries {
                    ui.horizontal(|ui| {
                        let name = entry.file_name();
                        ui.label(name.to_str().unwrap().to_owned());
                    });
                }
                // ui.horizontal(|ui| {
                //     ui.label("file 1");
                //     ui.label("10kb");
                // });
                // ui.horizontal(|ui| {
                //     ui.label("file 2");
                //     ui.label("100Gb");
                // });
            });

            ui.separator();

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
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
