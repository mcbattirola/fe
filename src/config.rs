use std::{fs, path::PathBuf};

use serde::Deserialize;

use crate::commands::Commands;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub commands: Option<Commands>,
    pub data_dir: Option<String>,
}

/// Represents the commands config containing file and directory commands.
#[derive(Debug, Deserialize)]
pub struct CommandsConfig {
    pub commands: Commands,
}

/// Reads and parses the configuration file from the given path, returning a Config.
pub fn parse_commands_config(file_path: &PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(file_path)?;
    let mut config: Config = toml::from_str(&config_str)?;

    let mut home = home::home_dir().unwrap();
    home.push(".fe");
    let fe_home = home.clone();

    if config.data_dir.is_none() {
        let mut data_dir = fe_home.clone();
        data_dir.push("data");
        config.data_dir = Some(data_dir.to_string_lossy().to_string());
    }

    println!("config: {:?}", config);
    Ok(config)
}
