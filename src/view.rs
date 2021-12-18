use crate::graphics::*;
use crate::text::*;
use crate::util::*;
use winit::event;

pub struct View {
    start: usize,
    dims: Rect,

    cursor_blink_on: bool,
    cursor_pos: Point2<u32>,

    points: Vec<CharBox>,
    block_types: Vec<BlockType>,
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

        let size = (dims.x * dims.y) as usize;
        let mut glyphs = Vec::with_capacity(size);
        let mut points = Vec::with_capacity(size);
        let mut block_types = Vec::with_capacity(size);

        for y in 0..dims.y {
            for x in 0..dims.x {
                points.push(pt(x, y));
                block_types.push(BlockType::Normal);
                glyphs.extend_from_slice(&glyph_list.glyphs);
            }
        }

        return Self {
            start: 0,
            dims,

            cursor_blink_on: true,
            cursor_pos: Point2 { x: 0, y: 0 },

            points,
            block_types,
            glyphs,
            did_raster: true,
        };
    }

    pub fn insert_char(&mut self, window: &Window, text: &mut File, c: char) {
        let mut s = [0u8; 4];
        let s = c.encode_utf8(&mut s);
        self.insert(window, text, s);
    }

    pub fn insert(&mut self, window: &Window, text: &mut File, s: &str) {
        self.cursor_blink_on = true;

        let start_line = match text.line_for_cursor(self.start) {
            Some(line) => line,
            None => {
                let line = text.last_line_begin();
                self.start = text.line_for_cursor(line).unwrap();

                line
            }
        };

        // match text.text_after_cursor(self.start) {}
        // flow_text(, self.dims, |pos, c| {});

        text.push_str(s);

        window.request_redraw();
    }

    pub fn toggle_cursor_blink(&mut self, window: &Window) {
        self.cursor_blink_on = !self.cursor_blink_on;
        window.request_redraw();
    }

    pub fn cursor_move(&mut self, window: &Window, key: event::VirtualKeyCode) -> bool {
        return self.v_cursor_move(window, key);
    }

    fn v_cursor_move(&mut self, window: &Window, key: event::VirtualKeyCode) -> bool {
        match key {
            event::VirtualKeyCode::Up => {
                if self.cursor_pos.y > 0 {
                    self.cursor_pos.y -= 1;
                }
            }
            event::VirtualKeyCode::Down => {
                if self.cursor_pos.y < self.dims.y - 1 {
                    self.cursor_pos.y += 1;
                }
            }
            event::VirtualKeyCode::Left => {
                if self.cursor_pos.x > 0 {
                    self.cursor_pos.x -= 1;
                }
            }
            event::VirtualKeyCode::Right => {
                if self.cursor_pos.x < self.dims.x - 1 {
                    self.cursor_pos.x += 1;
                }
            }
            _ => return false,
        }

        self.cursor_blink_on = true;
        window.request_redraw();
        return true;
    }

    pub fn draw(&mut self, text: &mut File, glyphs: &mut GlyphCache) {
        let cursor_pos = self.cursor_blink_on.then(|| self.cursor_pos);

        match text.text_after_cursor(self.start) {
            None => self.glyphs.fill(EMPTY_GLYPH),
            Some(text) => flow_text(text, self.dims, |pos, write_len, c| {
                let idx = (pos.y * self.dims.x + pos.x) as usize;

                if c.is_whitespace() {
                    self.glyphs[idx..(idx + write_len)].fill(EMPTY_GLYPH);
                    return;
                }

                let mut tmp = [0; 4];
                let c_str = c.encode_utf8(&mut tmp);
                let glyph_list = glyphs.translate_glyphs(c_str);
                self.did_raster = self.did_raster || glyph_list.did_raster;

                self.glyphs[idx..(idx + write_len)].fill(glyph_list.glyphs[0]);
            }),
        }

        // clear state
        for block_type in &mut self.block_types {
            *block_type = BlockType::Normal;
        }

        if self.cursor_blink_on {
            let idx = self.cursor_pos.y * self.dims.x + self.cursor_pos.x;
            self.block_types[idx as usize] = BlockType::Cursor;
        }

        let atlas = self.did_raster.then(|| glyphs.atlas());
        let atlas_dims = glyphs.atlas_dims();

        let result = TEXT_SHADER.with(|shader| -> Result<(), JsValue> {
            shader.render(
                atlas,
                &self.points,
                &self.block_types,
                &self.glyphs,
                atlas_dims,
                self.dims,
            )?;

            return Ok(());
        });

        expect(result);
    }
}

fn flow_text<'a, F>(text: impl Iterator<Item = &'a str>, dims: Rect, mut f: F)
where
    F: FnMut(Point2<u32>, usize, char),
{
    let mut place_char = |pos: &mut Point2<u32>, write_len: u32, c: char| -> bool {
        if pos.y == dims.y {
            return true;
        }

        f(*pos, write_len as usize, c);

        pos.x += write_len;
        if pos.x >= dims.x {
            pos.x = 0;
            pos.y += 1;
        }

        return false;
    };

    let mut pos = Point2 { x: 0, y: 0 };

    for text in text {
        for c in text.chars() {
            let len = match c {
                '\n' => dims.x - pos.x,
                '\t' => 2,
                c if c.is_control() => continue,
                _ => 1,
            };

            if place_char(&mut pos, len, c) {
                return;
            }
        }
    }

    while !place_char(&mut pos, 1, ' ') {}
}
