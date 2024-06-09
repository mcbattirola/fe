use std::process::Command;

// TODO: return errors
#[cfg(target_os = "windows")]
pub fn open_terminal(path: &str) {
    if let Err(e) = Command::new("cmd.exe")
        .args(&["/C", "start", "cmd.exe", "/K", &format!("cd /D {}", path)])
        .spawn()
    {
        eprintln!("Failed to open cmd.exe: {}", e);
    }
}

#[cfg(target_os = "macos")]
pub fn open_terminal(path: &str) {
    if let Err(e) = Command::new("open")
        .arg("-a")
        .arg("Terminal")
        .arg(path)
        .spawn()
    {
        eprintln!("Failed to open Terminal: {}", e);
    }
}

#[cfg(target_os = "linux")]
pub fn open_terminal(path: &str) {
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
        // TODO: st
    ];

    for (terminal, arg) in &terminals {
        if let Ok(_child) = Command::new(terminal).arg(arg).arg(path).spawn() {
            return;
        }
    }

    eprintln!("Failed to open any terminal emulator.");
}
