use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::process::{Command as ProcessCommand, Stdio};

/// Represents a command configuration.
#[derive(Debug, Deserialize)]
pub struct Command {
    pub name: String,
    pub interpreter: String,
    pub script: String,
    pub extensions: Option<Vec<String>>,
    pub args: Option<Vec<String>>,
}

impl Command {
    pub fn run(&self, target_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let mut command = ProcessCommand::new(&self.interpreter);

        // Add interpreter arguments if any
        if let Some(args) = &self.args {
            command.args(args);
        }

        // Add script path if provided and not empty
        if !self.script.is_empty() {
            command.arg(&self.script);
        }

        // Add the target file or directory as the final argument
        command.arg(target_path);

        // Run the command
        command
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .wait()?;
        Ok(())
    }
}

/// Represents the overall configuration containing file and directory commands.
#[derive(Debug, Deserialize)]
pub struct CommandsConfig {
    pub commands: Commands,
}

/// Contains file and directory commands.
#[derive(Debug, Deserialize)]
pub struct Commands {
    pub file: Option<Vec<Command>>,
    pub dir: Option<Vec<Command>>,
}

/// Reads and parses the configuration file from the given path, returning a CommandMap.
pub fn parse_commands_config(file_path: &PathBuf) -> Result<Commands, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(file_path)?;
    println!("config_str: {:?}", config_str);
    let commands_config: CommandsConfig = toml::from_str(&config_str)?;
    println!("commands_config: {:?}", commands_config);
    Ok(commands_config.commands)
}
