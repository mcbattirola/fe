mod fe;
use eframe::run_native;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };
    run_native("fe", native_options, Box::new(|cc| Box::new(fe::FE::new(cc))))
}
