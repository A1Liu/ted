use crate::graphics::*;
use crate::text::*;
use crate::util::*;
use crate::view::*;
use winit::event_loop::ControlFlow;
use winit::window::Window;

pub enum TedCommand<'a> {
    RequestRedraw,
    Draw,
    Exit,

    InsertText { index: usize, text: &'a str },
    AppendText { text: &'a str },
    DeleteText { begin: usize, end: usize },

    ForView { command: ViewCommand<'a> },
}

pub enum ViewCommand<'a> {
    CursorMove(Direction),
    ToggleCursorBlink,
    Insert { text: &'a str },
    Delete,
    FlowCursor { index: usize },
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn from_arrow_key(key: winit::event::VirtualKeyCode) -> Option<Self> {
        return match key {
            winit::event::VirtualKeyCode::Up => Some(Self::Up),
            winit::event::VirtualKeyCode::Down => Some(Self::Down),
            winit::event::VirtualKeyCode::Left => Some(Self::Left),
            winit::event::VirtualKeyCode::Right => Some(Self::Right),
            _ => None,
        };
    }
}

pub struct CommandHandler {
    // This should be global
    cache: GlyphCache,

    // These eventually should not be global
    view: View,
    file: File,
}

impl CommandHandler {
    pub fn new(text: String) -> Self {
        let mut cache = GlyphCache::new();
        let view = View::new(new_rect(28, 15), &mut cache);
        let mut file = File::new();
        file.push(&text);

        return Self { cache, view, file };
    }

    pub fn run(&mut self, window: &Window, flow: &mut ControlFlow, mut commands: Vec<TedCommand>) {
        loop {
            let queued: Vec<_> = commands.drain(..).collect();
            if queued.len() == 0 {
                break;
            }

            for command in queued {
                match command {
                    TedCommand::RequestRedraw => window.request_redraw(),
                    TedCommand::Draw => self.view.draw(&self.file, &mut self.cache),
                    TedCommand::Exit => *flow = ControlFlow::Exit,

                    TedCommand::InsertText { index, text } => self.file.insert(index, text),
                    TedCommand::AppendText { text } => self.file.push(text),
                    TedCommand::DeleteText { begin, end } => self.file.delete(begin, end),

                    TedCommand::ForView { command } => {
                        self.view.run(&self.file, command, &mut commands)
                    }
                }
            }
        }

        let mut cursor = 0;
        while cursor < commands.len() {}
    }
}
