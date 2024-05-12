use egui;

pub enum Modifier {
    _Alt,
    _Ctrl,
    _Shift,
    _MacCmd,
    Cmd,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandEvent {
    FocusPathBar,
    FocusSearchBar,
    _Quit,
    DirGoBack,
    FavoritePath,
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
            ]),
            events: Vec::new(),
        };
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        // flush last frame's events
        self.events = Vec::new();

        'cmd_loop: for cmd in &self.commands {
            // check keys
            for key in &cmd.key {
                if !ctx.input(|i| i.key_pressed(*key)) {
                    continue 'cmd_loop;
                }
            }

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
                    continue 'cmd_loop;
                }
                // emit
                println!("emiting event {:?}", cmd.event);
                self.events.push(cmd.event.clone())
            }
        }
    }

    // get returns true if `event` is in the event pool
    pub fn get_event(&self, event: CommandEvent) -> bool {
        return self.events.contains(&event);
    }
}
