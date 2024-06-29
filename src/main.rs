#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod cli;
mod commands;
mod config;
mod events;
mod fe;
mod storage;
mod utils;

fn main() -> eframe::Result<()> {
    let args = cli::parse_args();
    let fe = fe::FE::from_args(args);
    fe.run()
}
