use home;
use std::path::PathBuf;
use structopt::StructOpt;

/// Command-line arguments struct
#[derive(Debug, StructOpt)]
pub struct Config {
    /// Path to the configuration file
    #[structopt(long)]
    pub config_path: Option<PathBuf>,

    /// Path to the data dir parent
    #[structopt(long)]
    pub data_dir: Option<PathBuf>,
}

pub fn parse_args() -> Config {
    let mut args = Config::from_args();

    let mut default_dir = home::home_dir().unwrap();
    default_dir.push(".fe");

    if args.config_path.is_none() {
        let mut default_path = default_dir.clone();
        default_path.push("config.toml");
        args.config_path = Some(default_path);
    }
    if args.data_dir.is_none() {
        let mut default_path = default_dir.clone();
        default_path.push("data");
        args.data_dir = Some(default_path);
    }

    return args;
}
