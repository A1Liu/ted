use crate::text::*;

pub enum TedCommand<'a> {
    RequestRedraw,

    InsertText { index: usize, text: &'a str },
    DeleteText { begin: usize, end: usize },

    ForView { command: ViewCommand<'a> },
}

pub enum ViewCommand<'a> {
    CursorMove(Direction),
    Insert { file: &'a File, text: &'a str },
    Delete { file: &'a File },
    FlowCursor { file: &'a File, file_index: usize },
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

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum TedEvent {
    Tick(usize),
}
