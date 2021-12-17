use crate::util::*;

pub struct View {
    // eventually do something else here lol
    text: String,

    start: usize,
    dims: Rect,

    cursor_position: Option<Idx>,
    cursor_blink_on: bool,
    cursor_pos: Point2<u32>,
}

impl View {
    pub fn new(dims: Rect, text: String) -> Self {
        return Self {
            text,

            start: 0,
            dims,

            cursor_position: Some(Idx::new(0)),
            cursor_blink_on: true,
            cursor_pos: Point2 { x: 0, y: 0 },
        };
    }

    pub fn cursor_up(&mut self) {}
}
