use egui;

pub enum Modifier {
    Alt,
    Ctrl,
    Shift,
    MacCmd,
    Cmd,
}

pub struct Command {
    modifiers: Vec<Modifier>,
    key: Vec<egui::Key>,
    event: CommandEvent,
}

pub struct CommandPool {
    commands: Vec<Command>,
    events: Vec<CommandEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandEvent {
    FocusPathBar,
    FocusSearchBar,
}

impl CommandPool {
    pub fn new() -> Self {
        return Self {
            commands: Vec::from([
                Command {
                    modifiers: Vec::from([Modifier::Cmd]),
                    key: Vec::from([egui::Key::L]),
                    event: CommandEvent::FocusPathBar,
                },
                Command {
                    modifiers: Vec::from([Modifier::Cmd]),
                    key: Vec::from([egui::Key::F]),
                    event: CommandEvent::FocusSearchBar,
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
                        Modifier::Alt => i.modifiers.alt,
                        Modifier::Ctrl => i.modifiers.ctrl,
                        Modifier::Shift => i.modifiers.shift,
                        Modifier::MacCmd => i.modifiers.mac_cmd,
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
