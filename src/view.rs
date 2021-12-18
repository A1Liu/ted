use crate::graphics::*;
use crate::text::*;
use crate::util::*;

#[inline]
fn pt(x: u32, y: u32, block_type: u32) -> Vector3<u32> {
    let z = block_type;
    return Vector3 { x, y, z };
}

pub struct View {
    start: usize,
    dims: Rect,

    cursor_position: Option<Idx>,
    cursor_blink_on: bool,
    cursor_pos: Point2<u32>,

    points: Vec<Vector3<u32>>,
    glyphs: Vec<Glyph>,
    did_raster: bool,
}

struct TextFlow<'a> {
    pos: Point2<u32>,
    cache: &'a mut GlyphCache,
}

impl View {
    pub fn new(dims: Rect, cache: &mut GlyphCache) -> Self {
        // TODO This assumes characters are 1 glyph
        let glyph_list = cache.translate_glyphs(" ");
        let did_raster = glyph_list.did_raster;

        let size = (dims.x * dims.y) as usize;
        let mut glyphs = Vec::with_capacity(size);
        let mut points = Vec::with_capacity(size * 6);

        for y in 0..dims.y {
            for x in 0..dims.x {
                points.extend_from_slice(&[
                    pt(x, y, 0),
                    pt(x + 1, y, 0),
                    pt(x, y + 1, 0),
                    pt(x, y + 1, 0),
                    pt(x + 1, y, 0),
                    pt(x + 1, y + 1, 0),
                ]);

                glyphs.extend_from_slice(&glyph_list.glyphs);
            }
        }

        return Self {
            start: 0,
            dims,

            cursor_position: Some(Idx::new(0)),
            cursor_blink_on: true,
            cursor_pos: Point2 { x: 0, y: 0 },

            points,
            glyphs,
            did_raster,
        };
    }

    pub fn insert_char(&mut self, window: &Window, text: &mut File, c: char) {
        let mut s = String::new();
        s.push(c);
        self.insert(window, text, s);
    }

    pub fn insert(&mut self, window: &Window, text: &mut File, s: String) {
        self.cursor_blink_on = true;

        // TODO move text flowing here from the text vertices thing, so that
        // we can determine cursor location after insertion

        text.push_str(&s);
        window.request_redraw();
    }

    pub fn toggle_cursor_blink(&mut self, window: &Window) {
        self.cursor_blink_on = !self.cursor_blink_on;
        window.request_redraw();
    }

    pub fn cursor_up(&mut self, window: &Window) {
        self.cursor_blink_on = true;
        if self.cursor_pos.y > 0 {
            self.cursor_pos.y -= 1;
        }

        window.request_redraw();
    }

    pub fn cursor_left(&mut self, window: &Window) {
        self.cursor_blink_on = true;
        if self.cursor_pos.x > 0 {
            self.cursor_pos.x -= 1;
        }

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

    fn rewrite_cursor(&mut self) {
        // clear state
        for point in &mut self.points {
            point.z = 0;
        }

        // For z:
        // 0 is normal
        // 1 is cursor
        // 2 is selected
        // Set the block mode for the points that represent the cursor
        if self.cursor_blink_on {
            let idx = self.cursor_pos.y * self.dims.x + self.cursor_pos.x;
            let idx = (idx * 6) as usize;

            for idx in idx..(idx + 6) {
                self.points[idx].z = 1;
            }
        }
    }

    fn flow_text(&mut self, cache: &mut GlyphCache, text: &File) {
        let pos = Point2 { x: 0, y: 0 };
        let mut flow = TextFlow { cache, pos };

        if let Some(text) = text.text_after_cursor(self.start) {
            for text in text {
                for c in text.chars() {
                    if c == '\n' {
                        let len = self.dims.x - flow.pos.x;
                        if self.place_char(&mut flow, len, ' ') {
                            return;
                        }

                        continue;
                    }

                    if c == '\t' {
                        if self.place_char(&mut flow, 2, ' ') {
                            return;
                        }

                        continue;
                    }

                    if c.is_whitespace() {
                        if self.place_char(&mut flow, 1, ' ') {
                            return;
                        }

                        continue;
                    }

                    if c.is_control() {
                        continue;
                    }

                    if self.place_char(&mut flow, 1, c) {
                        return;
                    }
                }
            }
        }

        while !self.place_char(&mut flow, 1, ' ') {}
    }

    fn place_char(&mut self, flow: &mut TextFlow, repeat: u32, c: char) -> bool {
        let mut tmp = [0; 4];
        let c_str = c.encode_utf8(&mut tmp);

        // TODO This assumes characters are 1 glyph
        let glyph_list = flow.cache.translate_glyphs(c_str);
        self.did_raster = self.did_raster || glyph_list.did_raster;

        let mut write_len = repeat;

        let mut pos = flow.pos;
        for y in pos.y..self.dims.y {
            for x in pos.x..self.dims.x {
                if write_len == 0 {
                    flow.pos = Point2 { x, y };
                    return false;
                }

                // we write!
                let idx = (y * self.dims.x + x) as usize;
                self.glyphs[idx] = glyph_list.glyphs[0];
                write_len -= 1;
            }

            pos.x = 0;
            write_len = 0; // Don't do any wrapping work here
        }

        flow.pos = self.dims.into();
        return true;
    }

    pub fn draw(&mut self, text: &mut File, glyphs: &mut GlyphCache) {
        let cursor_pos = self.cursor_blink_on.then(|| self.cursor_pos);

        self.flow_text(glyphs, text);
        self.rewrite_cursor();

        let atlas = self.did_raster.then(|| glyphs.atlas());
        let atlas_dims = glyphs.atlas_dims();

        let result = TEXT_SHADER.with(|shader| -> Result<(), JsValue> {
            shader.render(atlas, &self.points, &self.glyphs, atlas_dims, self.dims)?;

            return Ok(());
        });

        expect(result);
    }
}
