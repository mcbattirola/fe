use std::{fs, path::Path, path::PathBuf};

use serde::Deserialize;

use crate::commands::Commands;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub commands: Option<Commands>,
    pub data_dir: Option<String>,
}

/// Reads and parses the configuration file from the given path, returning a Config.
pub fn parse_config(file_path: &PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
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

    // iterate over commands and make all absolute path
    if let Some(cmds) = &mut config.commands {
        let base_path = file_path.parent().unwrap_or(Path::new("/"));

        for file_cmds in &mut cmds.file {
            for cmd in file_cmds {
                let script_path = Path::new(&cmd.script);
                if !script_path.is_absolute() {
                    let absolute_path = base_path.join(script_path);
                    cmd.script = absolute_path.to_string_lossy().into_owned();
                }
            }
        }
        for dir_cmds in &mut cmds.dir {
            for cmd in dir_cmds {
                let script_path = Path::new(&cmd.script);
                if !script_path.is_absolute() {
                    let absolute_path = base_path.join(script_path);
                    cmd.script = absolute_path.to_string_lossy().into_owned();
                }
            }
        }
    }

    println!("config: {:?}", config);
    Ok(config)
}
