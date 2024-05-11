mod command;
mod fe;

fn main() -> eframe::Result<()> {
    let fe = fe::FE::new();
    fe.run()
}
