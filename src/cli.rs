use home;
use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
};
use structopt::StructOpt;

/// Command-line arguments struct
#[derive(Debug, StructOpt)]
pub struct CliArgs {
    /// Path to the configuration file
    #[structopt(long)]
    pub config_path: Option<PathBuf>,
}

pub fn parse_args() -> CliArgs {
    let mut args = CliArgs::from_args();

    if args.config_path.is_none() {
        let mut default_path = get_fe_dir();
        default_path.push("config.toml");
        ensure_default(&default_path);
        args.config_path = Some(default_path);
    }
    return args;
}

pub fn ensure_default(default_config: &PathBuf) {
    println!("creating {:?}", default_config);

    // Create parent directories if they don't exist
    if let Some(parent) = default_config.parent() {
        fs::create_dir_all(parent).expect("can't create config dir");
    }

    // Open the file in write mode, create it if it doesn't exist
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(default_config);

    match file {
        Ok(_) => println!("Config file created: {:?}", default_config),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            println!("Config file already exists: {:?}", default_config)
        }
        Err(e) => panic!("couldn't create config file: {:?}", e),
    }
}

pub fn get_fe_dir() -> PathBuf {
    // defaults to $HOME/.config/fe/config.toml
    // TODO: respect XDG_CONFIG_HOME
    let mut default_dir = home::home_dir().unwrap();
    default_dir.push(".config/fe");
    return default_dir;
}
