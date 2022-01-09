use crate::util::*;
use crate::view::*;

#[cfg_attr(debug_assertions, derive(PartialEq))]
pub enum TedCommand {
    DrawView {
        is_lines: bool,
        block_types: Vec<BlockType>,
        ranges: Vec<RangeData>,
        text: Vec<char>,
        dims: Rect,
    },

    RequestRedraw,
    Exit,

    ForView {
        command: ViewCommand,
    },
}

#[inline(always)]
pub fn for_view(command: ViewCommand) -> TedCommand {
    return TedCommand::ForView { command };
}

#[cfg_attr(debug_assertions, derive(PartialEq))]
pub enum ViewCommand {
    CursorMove(Direction),
    ToggleCursorBlink,
    Insert { text: String },
    DeleteAfterCursor,
    FlowCursor { index: usize },
    SetContents(SetContents),
    Draw,
}

#[cfg_attr(debug_assertions, derive(PartialEq))]
pub struct SetContents {
    pub start: usize,
    pub start_line: usize,
    pub text: String,
}

#[cfg_attr(debug_assertions, derive(PartialEq))]
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

pub struct Command<'a, Value> {
    pub buffer: &'a mut Vec<TedCommand>,
    pub value: Value,
}
