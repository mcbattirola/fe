mod command;
mod fe;
mod storage;
mod utils;

fn main() -> eframe::Result<()> {
    let fe = fe::FE::new();
    fe.run()
}
