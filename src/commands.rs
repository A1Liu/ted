use crate::graphics::*;
use crate::util::*;
use crate::view::*;
use winit::event_loop::ControlFlow;
use winit::window::Window;

pub enum TedCommand {
    DrawView {},
    RequestRedraw,
    Exit,

    ForView { command: ViewCommand },
}

#[inline(always)]
pub fn for_view(command: ViewCommand) -> TedCommand {
    return TedCommand::ForView { command };
}

pub enum ViewCommand {
    CursorMove(Direction),
    ToggleCursorBlink,
    Insert { text: String },
    DeleteAfterCursor,
    FlowCursor { index: usize },
    SetContents(SetContents),
    Draw,
}

pub struct SetContents {
    pub start: usize,
    pub start_line: usize,
    pub text: String,
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

    // This eventually should not be global
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

    pub fn run(&mut self, window: &Window, flow: &mut ControlFlow, command: TedCommand) {
        let mut commands = Vec::with_capacity(8);
        let mut buffer = &mut Vec::with_capacity(8);

        commands.push(command);

        while let Some(command) = commands.pop() {
            match command {
                TedCommand::RequestRedraw => window.request_redraw(),
                TedCommand::Exit => *flow = ControlFlow::Exit,
                TedCommand::ForView {
                    command: ViewCommand::Draw,
                }
                | TedCommand::DrawView {} => self.view.draw(&mut self.cache, buffer),
                TedCommand::ForView { command } => {
                    let cmd = Command {
                        buffer,
                        value: command,
                    };

                    self.view.run(cmd)
                }
            }

            commands.extend(buffer.drain(..).rev());
        }
    }
}

pub struct Command<'a, Value> {
    pub buffer: &'a mut Vec<TedCommand>,
    pub value: Value,
}
