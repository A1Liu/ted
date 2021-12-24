use crate::text::*;

pub enum TedCommand<'a> {
    RequestRedraw,
    Exit,

    InsertText { index: usize, text: &'a str },
    AppendText { text: &'a str },
    DeleteText { begin: usize, end: usize },

    ForView { command: ViewCommand<'a> },
}

pub enum ViewCommand<'a> {
    CursorMove(Direction),
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
