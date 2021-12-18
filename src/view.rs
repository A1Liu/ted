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

    pub fn insert_char(&mut self, window: &Window, file: &mut File, c: char) {
        let mut s = [0u8; 4];
        let s = c.encode_utf8(&mut s);
        self.insert(window, file, s);
    }

    pub fn insert(&mut self, window: &Window, file: &mut File, s: &str) {
        self.cursor_blink_on = true;

        if s.len() == 0 {
            window.request_redraw();
            return;
        }

        let start_line = file.line_for_cursor(self.start).unwrap();
        let text = file.text_after_cursor(self.start).unwrap();

        // @Memory we can probably reduce the number of fields used here. Not
        // all of these are necessary
        //                          - Albert Liu, Dec 18, 2021 Sat 14:31 EST
        #[derive(Debug)]
        enum FlowResult {
            NotFound,
            Found {
                index: usize,
            },

            // Visual line, not textual
            FoundLineBegin {
                begin: usize,
            },

            // Visual line, not textual
            FoundLine {
                pos: Point2<u32>,
                begin: usize,
                end: usize,
            },
        }

        let mut result = FlowResult::NotFound;
        let flow = flow_text(text, self.dims, |state, write_len, c| {
            if state.pos == self.cursor_pos {
                result = FlowResult::Found { index: state.index };
                return;
            }

            match &mut result {
                FlowResult::Found { .. } => return,
                FlowResult::FoundLine { .. } => return,
                r @ FlowResult::NotFound => {
                    if state.pos.y == self.cursor_pos.y {
                        *r = FlowResult::FoundLineBegin { begin: state.index };
                        if c == '\n' {
                            let (pos, begin, end) = (state.pos, state.index, state.index);
                            *r = FlowResult::FoundLine { pos, begin, end };
                        }
                    }
                }
                FlowResult::FoundLineBegin { begin } => {
                    if c == '\n' {
                        let (pos, begin, end) = (state.pos, *begin, state.index);
                        result = FlowResult::FoundLine { pos, begin, end };
                        return;
                    }
                }
            }
        });

        if flow.pos == self.cursor_pos {
            result = FlowResult::Found { index: flow.index };
        }

        let index = match result {
            FlowResult::Found { index } => index,
            FlowResult::FoundLine { pos, begin, end } => {
                let mut index = begin + pos.x as usize;
                if s.chars().nth(0).unwrap() != '\n' {
                    for x in pos.x..self.cursor_pos.x {
                        file.insert(end, '~');
                        index += 1;
                    }
                }

                index
            }
            FlowResult::FoundLineBegin { begin } => {
                let mut index = begin + flow.pos.x as usize;

                if s.chars().nth(0).unwrap() != '\n' {
                    for x in flow.pos.x..self.cursor_pos.x {
                        file.push('~');
                        index += 1;
                    }
                    index = begin + self.cursor_pos.x as usize;
                }

                index
            }
            FlowResult::NotFound => {
                let mut index = flow.index;
                for y in flow.pos.y..self.cursor_pos.y {
                    file.push('\n');
                    index += 1;
                }

                if s.chars().nth(0).unwrap() != '\n' {
                    for x in 0..self.cursor_pos.x {
                        file.push('~');
                        index += 1;
                    }
                }

                index
            }
        };

        let text_index = self.start + index;
        file.insert_str(text_index, s);

        let count = s.chars().count();
        let text = file.text_after_cursor(self.start).unwrap();
        let mut next_pos = None;
        let flow = flow_text(text, self.dims, |state, write_len, c| {
            if state.index == index + count {
                next_pos = Some(state.pos);
            }
        });

        if flow.index == index + count && !flow.is_full {
            next_pos = Some(flow.pos);
        }

        // TODO typing at the end of the screen
        if let Some(pos) = next_pos {
            self.cursor_pos = pos;
        }

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

        let mut pt = match text.text_after_cursor(self.start) {
            None => Point2 { x: 0, y: 0 },
            Some(text) => {
                let state = flow_text(text, self.dims, |state, write_len, c| {
                    let idx = (state.pos.y * self.dims.x + state.pos.x) as usize;

                    if c.is_whitespace() {
                        self.glyphs[idx..(idx + write_len)].fill(EMPTY_GLYPH);
                        return;
                    }

                    let mut tmp = [0; 4];
                    let c_str = c.encode_utf8(&mut tmp);
                    let glyph_list = glyphs.translate_glyphs(c_str);
                    self.did_raster = self.did_raster || glyph_list.did_raster;

                    self.glyphs[idx..(idx + write_len)].fill(glyph_list.glyphs[0]);
                });

                state.pos
            }
        };

        for y in pt.y..self.dims.y {
            let len = (self.dims.x - pt.x) as usize;
            let idx = (y * self.dims.x + pt.x) as usize;

            self.glyphs[idx..(idx + len)].fill(EMPTY_GLYPH);
            pt.x = 0;
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

#[derive(Clone, Copy)]
struct FlowState {
    is_full: bool,
    pos: Point2<u32>,
    index: usize,
}

fn flow_text<'a, F>(text: impl Iterator<Item = &'a str>, dims: Rect, mut f: F) -> FlowState
where
    F: FnMut(FlowState, usize, char),
{
    let mut place_char = |state: &mut FlowState, write_len: u32, c: char| {
        if state.pos.y >= dims.y {
            state.is_full = true;
        }

        f(*state, write_len as usize, c);

        state.pos.x += write_len;
        if state.pos.x >= dims.x {
            state.pos.x = 0;
            state.pos.y += 1;
        }

        state.is_full = state.pos.y >= dims.y;
    };

    let mut state = FlowState {
        is_full: false,
        pos: Point2 { x: 0, y: 0 },
        index: 0,
    };

    for text in text {
        for c in text.chars() {
            let len = match c {
                '\n' => dims.x - state.pos.x,
                '\t' => 2,
                c if c.is_control() => {
                    state.index += 1;
                    continue;
                }
                _ => 1,
            };

            place_char(&mut state, len, c);
            state.index += 1;
            if state.is_full {
                return state;
            }
        }
    }

    return state;
}
