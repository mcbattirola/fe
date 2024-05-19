mod command;
mod fe;
mod utils;

fn main() -> eframe::Result<()> {
    let fe = fe::FE::new();
    fe.run()
}
