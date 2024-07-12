use serde::Deserialize;
use std::path::PathBuf;
use std::process::{Command as ProcessCommand, Stdio};

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct FileCommand {
    pub name: String,
    pub interpreter: String,
    pub script: String,
    pub extensions: Option<Vec<String>>,
    pub args: Option<Vec<String>>,
    pub clickable: Option<bool>,
}

/// Represents a command configuration.
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct DirCommand {
    pub name: String,
    pub interpreter: String,
    pub script: String,
    pub args: Option<Vec<String>>,
}

impl FileCommand {
    pub fn run(&self, target_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        return run(&self.interpreter, &self.script, &self.args, target_path);
    }
}

impl DirCommand {
    pub fn run(&self, target_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        return run(&self.interpreter, &self.script, &self.args, target_path);
    }
}

/// Contains file and directory commands.
#[derive(Debug, Deserialize)]
pub struct Commands {
    pub file: Option<Vec<FileCommand>>,
    pub dir: Option<Vec<DirCommand>>,
}

fn run(
    interpreter: &String,
    script: &String,
    args: &Option<Vec<String>>,
    target_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = ProcessCommand::new(interpreter);

    if let Some(args) = args {
        command.args(args);
    }

    if !script.is_empty() {
        command.arg(script);
    }

    // Add the target file or directory as the final argument
    command.arg(target_path);

    // Platform-specific configurations for detaching process
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            command.pre_exec(|| Ok(()));
            libc::setsid();
        }
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(winapi::um::winbase::CREATE_NEW_PROCESS_GROUP);
    }

    command
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    Ok(())
}
