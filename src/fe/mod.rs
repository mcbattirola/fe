pub struct FE {
    // Example stuff:
    label: String,
}

impl FE {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        // Default::default()
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
        }
    }
}

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
        let _path_resp = egui::TopBottomPanel::top("pathbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Path");
                ui.text_edit_singleline(&mut self.label);
            });
        }).response;

        // hover example:
        // print!("hovered? {:?}\n", _path_resp.hovered());

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Files");

            ui.vertical_centered_justified(|ui| {
                ui.horizontal(|ui| {
                    ui.label("file 1");
                    ui.label("10kb");
                });
                ui.horizontal(|ui| {
                    ui.label("file 2");
                    ui.label("100Gb");
                });
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
