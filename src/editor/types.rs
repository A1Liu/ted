use crate::util::*;
use mint::*;

pub type Color = Vector3<f32>;

pub const fn color(r: f32, g: f32, b: f32) -> Color {
    return Color { x: r, y: g, z: b };
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Glyph {
    // TODO translate these to f32's maybe? Then we wouldn't need to save atlas
    // dims in a uniform either.
    //                          - Albert Liu, Jan 11, 2022 Tue 21:46 EST

    // each glyph is 2 trianges of 3 points each
    pub top_left_1: Point2<u32>,
    pub top_right_1: Point2<u32>,
    pub bot_left_1: Point2<u32>,
    pub top_right_2: Point2<u32>,
    pub bot_left_2: Point2<u32>,
    pub bot_right_2: Point2<u32>,
}

pub struct TextShaderInput<'a> {
    pub is_lines: bool,
    pub atlas: Option<&'a [u8]>,
    pub fg_colors: Pod<Color>,
    pub bg_colors: Pod<Color>,
    pub glyphs: Pod<Glyph>,
    pub atlas_dims: Rect,
    pub dims: Rect,
}

pub struct HLData {
    pub color: Pod<Color>,
    pub background: Pod<Color>,
}

#[derive(Clone, Copy)]
pub enum HLAction {
    BeginScope(usize),
    EndScope,
    None,
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

pub struct Command<'a, Value> {
    pub buffer: &'a mut Vec<TedCommand>,
    pub value: Value,
}
