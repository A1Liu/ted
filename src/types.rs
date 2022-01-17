use crate::util::CopyRange;
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
