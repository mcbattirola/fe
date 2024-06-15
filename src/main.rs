mod cli;
mod commands;
mod events;
mod fe;
mod storage;
mod utils;

fn main() -> eframe::Result<()> {
    let args = cli::parse_args();
    let fe = fe::FE::new(args);
    fe.run()
}
