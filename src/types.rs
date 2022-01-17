use crate::util::*;
use mint::*;

pub type Color = Vector3<f32>;

pub const fn color(r: f32, g: f32, b: f32) -> Color {
    return Color { x: r, y: g, z: b };
}

#[derive(Clone, Copy)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Style {
    pub fg_color: Color,
    pub bg_color: Color,
}

#[derive(Clone, Copy)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct RangeData {
    pub range: CopyRange,
    pub style: Style,
}

#[cfg_attr(debug_assertions, derive(PartialEq))]
pub enum TedCommand {
    DrawView {
        is_lines: bool,
        fg_colors: Pod<Color>,
        bg_colors: Pod<Color>,
        text: Pod<char>,
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
