use crate::util::*;

pub struct View {
    // eventually do something else here lol
    text: String,

    view_start: usize,
    view_dims: Rect,

    cursor_position: Option<Idx>,
    cursor_blink_on: bool,
    cursor_pos: Point2<u32>,
}
