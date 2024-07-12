use egui::{self, DroppedFile};

use crate::commands::{DirCommand, FileCommand};
use crate::utils;

#[derive(Debug)]
pub enum Modifier {
    _Alt,
    _Ctrl,
    _Shift,
    _MacCmd,
    Cmd,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventType {
    FocusPathBar,
    FocusSearchBar,
    DirGoBack,
    FavoriteCurrentPath,
    NewFile,
    OpenTerminal,
    SetPath(std::path::PathBuf),
    DeleteFile(utils::dir::FeEntry),
    Exec(std::path::PathBuf),
    RunDirCmd(DirCommand),
    RunFileCmd(FileCommand, std::path::PathBuf),
    MoveFile(u8, Vec<DroppedFile>),
    ReloadDir,
    _Quit,
}

pub struct Event {
    event: EventType,
    modifiers: Vec<Modifier>,
    key: Vec<egui::Key>,
}

pub struct EventPool {
    commands: Vec<Event>,
    events: Vec<EventType>,
    next_frame_events: Vec<EventType>,
}

impl EventPool {
    pub fn new() -> Self {
        return Self {
            commands: Vec::from([
                Event {
                    event: EventType::FocusPathBar,
                    modifiers: Vec::from([Modifier::Cmd]),
                    key: Vec::from([egui::Key::L]),
                },
                Event {
                    event: EventType::FocusSearchBar,
                    modifiers: Vec::from([Modifier::Cmd]),
                    key: Vec::from([egui::Key::F]),
                },
                Event {
                    event: EventType::DirGoBack,
                    modifiers: Vec::from([Modifier::Cmd]),
                    key: Vec::from([egui::Key::O]),
                },
                Event {
                    event: EventType::FavoriteCurrentPath,
                    modifiers: Vec::from([Modifier::Cmd]),
                    key: Vec::from([egui::Key::B]),
                },
                Event {
                    event: EventType::NewFile,
                    modifiers: Vec::from([Modifier::Cmd]),
                    key: Vec::from([egui::Key::N]),
                },
                Event {
                    event: EventType::ReloadDir,
                    modifiers: Vec::from([Modifier::Cmd]),
                    key: Vec::from([egui::Key::R]),
                },
            ]),
            events: Vec::new(),
            next_frame_events: Vec::new(),
        };
    }

    // clear the events and sends the events scheduled for the next frame
    pub fn flush_events(&mut self) {
        self.events = self.next_frame_events.clone();
        self.next_frame_events = Vec::new();
    }

    // reads the inputs and emits events for issued commands.
    // Should be called before handling events each frame.
    pub fn emit_input_events(&mut self, ctx: &egui::Context) {
        // flush last frame's events
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
                    println!("didn't found {:?}", cmd.modifiers);
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
    pub fn get_event(&self, event: EventType) -> bool {
        return self.events.contains(&event);
    }

    pub fn get_events(&self) -> Vec<EventType> {
        return self.events.clone();
    }

    pub fn emit_event(&mut self, event: EventType) {
        println!("emiting event {:?}", event);
        self.events.push(event);
    }

    // schedules an event to be emited next time the events are flushed
    pub fn schedule_event(&mut self, event: EventType) {
        self.next_frame_events.push(event);
    }
}
