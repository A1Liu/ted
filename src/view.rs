use crate::graphics::*;
use crate::text::*;
use crate::util::*;

pub struct View {
    // eventually do something else here lol
    text: File,

    start: usize,
    dims: Rect,

    cursor_position: Option<Idx>,
    cursor_blink_on: bool,
    cursor_pos: Point2<u32>,
}

impl View {
    pub fn new(dims: Rect, text: String) -> Self {
        let mut file = File::new();
        file.push_str(&text);

        return Self {
            text: file,

            start: 0,
            dims,

            cursor_position: Some(Idx::new(0)),
            cursor_blink_on: true,
            cursor_pos: Point2 { x: 0, y: 0 },
        };
    }

    pub fn insert_char(&mut self, window: &Window, c: char) {
        let mut s = String::new();
        s.push(c);
        self.insert(window, s);
    }

    pub fn insert(&mut self, window: &Window, s: String) {
        self.cursor_blink_on = true;
        self.text.push_str(&s);
        window.request_redraw();
    }

    pub fn toggle_cursor_blink(&mut self, window: &Window) {
        self.cursor_blink_on = !self.cursor_blink_on;
        window.request_redraw();
    }

    pub fn cursor_up(&mut self, window: &Window) {
        self.cursor_blink_on = true;
        self.cursor_pos.y = self.cursor_pos.y.saturating_sub(1);

        window.request_redraw();
    }

    pub fn cursor_left(&mut self, window: &Window) {
        self.cursor_blink_on = true;
        self.cursor_pos.x = self.cursor_pos.x.saturating_sub(1);

        window.request_redraw();
    }

    pub fn cursor_right(&mut self, window: &Window) {
        self.cursor_blink_on = true;
        if self.cursor_pos.x < self.dims.x - 1 {
            self.cursor_pos.x += 1;
        }

        window.request_redraw();
    }

    pub fn cursor_down(&mut self, window: &Window) {
        self.cursor_blink_on = true;
        if self.cursor_pos.y < self.dims.y - 1 {
            self.cursor_pos.y += 1;
        }

        window.request_redraw();
    }

    pub fn draw(&self, glyphs: &mut GlyphCache) {
        let cursor_pos = match self.cursor_blink_on {
            true => Some(self.cursor_pos),
            false => None,
        };

        let mut vertices = TextVertices::new(glyphs, self.dims, cursor_pos);
        if let Some(text) = self.text.text_after_cursor(self.start) {
            for text in text {
                vertices.push(text);
            }
        }

        expect(vertices.render());
    }
}
