use std::process::Command;

#[cfg(target_os = "windows")]
pub fn open_terminal(path: &str) -> Option<std::io::Error> {
    if let Err(e) = Command::new("cmd.exe")
        .args(&["/C", "start", "cmd.exe", "/K", &format!("cd /D {}", path)])
        .spawn()
    {
        return Some(e);
    }
    return None;
}

#[cfg(target_os = "macos")]
pub fn open_terminal(path: &str) -> Option<std::io::Error> {
    if let Err(e) = Command::new("open")
        .arg("-a")
        .arg("Terminal")
        .arg(path)
        .spawn()
    {
        return Some(e);
    }
    return None;
}

#[cfg(target_os = "linux")]
use std::io::{Error, ErrorKind};
#[cfg(target_os = "linux")]
pub fn open_terminal(path: &str) -> Option<std::io::Error> {
    // TODO: this whole thing needs to be improved.
    // Allacrity, for instance, should have --hold to persist
    // if we close the app.
    let terminals = [
        // tested
        ("alacritty", "--working-directory"),
        // untested
        ("gnome-terminal", "--working-directory"),
        ("konsole", "--workdir"),
        ("xterm", "-e"),
        ("terminator", "-p"),
        ("urxvt", "-cd"),
        // TODO:
        // - st
    ];

    for (terminal, arg) in &terminals {
        if let Ok(_child) = Command::new(terminal).arg(arg).arg(path).spawn() {
            return None;
        }
    }

    Some(Error::new(ErrorKind::Other, "failed to open terminal"))
}
