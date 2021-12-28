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

    ForView { command: ViewCommand<'a> },
}

#[inline(always)]
pub fn for_view<'a>(command: ViewCommand<'a>) -> TedCommand<'a> {
    return TedCommand::ForView { command };
}

pub enum ViewCommand<'a> {
    CursorMove(Direction),
    ToggleCursorBlink,
    Insert { text: &'a str },
    Delete,
    FlowCursor { index: usize },
    SetContents { start: usize, text: &'a str },
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

// These will ultimately be used to some simple form of RPC. Additionally, structuring
// the code like this should in theory improve testability for command-driven
// components.
pub struct CommandHandler {
    // This should be global
    cache: GlyphCache,

    // These eventually should not be global
    view: View,
}

// TODO this should use a stack instead of a queue, and also the pattern of passing
// a mutable Vec might not be the best once the switch to a stack happens. Maybe
// just return a Vec?
impl CommandHandler {
    pub fn new(text: String) -> Self {
        let mut cache = GlyphCache::new();
        let mut view = View::new(new_rect(35, 20), &text);

        return Self { cache, view };
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
                    TedCommand::Draw => self.view.draw(&mut self.cache),
                    TedCommand::Exit => *flow = ControlFlow::Exit,

                    TedCommand::ForView { command } => self.view.run(command, &mut commands),
                }
            }
        }
    }
}
