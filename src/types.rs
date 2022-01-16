use mint::*;

pub type Color = Vector3<f32>;

pub const fn color(r: f32, g: f32, b: f32) -> Color {
    return Color { x: r, y: g, z: b };
}

#[derive(Clone, Copy)]
pub struct CopyRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Copy)]
pub struct Style {
    pub fg_color: Color,
    pub bg_color: Option<Color>,
}

#[derive(Clone, Copy)]
pub struct RangeData {
    pub offset_from_last: usize,
    pub len: usize,
    pub style: Style,
}
