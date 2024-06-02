use egui;

#[derive(Debug)]
pub enum Modifier {
    _Alt,
    _Ctrl,
    _Shift,
    _MacCmd,
    Cmd,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandEvent {
    FocusPathBar,
    FocusSearchBar,
    DirGoBack,
    FavoritePath,
    NewFile,
    OpenTerminal,
    SetPath(std::path::PathBuf),
    DeleteFile(std::path::PathBuf),
    _Quit,
}

pub struct Command {
    event: CommandEvent,
    modifiers: Vec<Modifier>,
    key: Vec<egui::Key>,
}

pub struct CommandPool {
    commands: Vec<Command>,
    events: Vec<CommandEvent>,
}

impl CommandPool {
    pub fn new() -> Self {
        return Self {
            commands: Vec::from([
                Command {
                    event: CommandEvent::FocusPathBar,
                    modifiers: Vec::from([Modifier::Cmd]),
                    key: Vec::from([egui::Key::L]),
                },
                Command {
                    event: CommandEvent::FocusSearchBar,
                    modifiers: Vec::from([Modifier::Cmd]),
                    key: Vec::from([egui::Key::F]),
                },
                Command {
                    event: CommandEvent::DirGoBack,
                    modifiers: Vec::from([Modifier::Cmd]),
                    key: Vec::from([egui::Key::O]),
                },
                Command {
                    event: CommandEvent::FavoritePath,
                    modifiers: Vec::from([Modifier::Cmd]),
                    key: Vec::from([egui::Key::B]),
                },
                Command {
                    event: CommandEvent::NewFile,
                    modifiers: Vec::from([Modifier::Cmd]),
                    key: Vec::from([egui::Key::N]),
                },
            ]),
            events: Vec::new(),
        };
    }

    // reads the inputs and emits events for issued commands.
    // Should be called before handling events each frame.
    pub fn emit_input_events(&mut self, ctx: &egui::Context) {
        // flush last frame's events
        self.events = Vec::new();
        let mut events_to_emit = Vec::new();

        'cmd_loop: for cmd in &self.commands {
            // check keys
            for key in &cmd.key {
                if !ctx.input(|i| i.key_pressed(*key)) {
                    continue 'cmd_loop;
                }
            }
            println!("found {:?}", cmd.key);

            // check modifiers
            for m in &cmd.modifiers {
                if !ctx.input(|i| {
                    return match &m {
                        Modifier::_Alt => i.modifiers.alt,
                        Modifier::_Ctrl => i.modifiers.ctrl,
                        Modifier::_Shift => i.modifiers.shift,
                        Modifier::_MacCmd => i.modifiers.mac_cmd,
                        Modifier::Cmd => i.modifiers.command,
                    };
                }) {
                    println!("dodnt found {:?}", cmd.modifiers);
                    continue 'cmd_loop;
                }
            }
            events_to_emit.push(cmd.event.clone());
        }

        // emit collected events
        for event in events_to_emit {
            self.emit_event(event);
        }
    }

    // returns whether an event of the `event` type was emited
    pub fn get_event(&self, event: CommandEvent) -> bool {
        return self.events.contains(&event);
    }

    pub fn get_events(&self) -> Vec<CommandEvent> {
        return self.events.clone();
    }

    pub fn emit_event(&mut self, event: CommandEvent) {
        println!("emiting event {:?}", event);
        self.events.push(event);
    }
}
