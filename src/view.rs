use crate::commands::*;
use crate::graphics::*;
use crate::text::*;
use crate::util::*;
use winit::event;

pub struct View {
    start: usize,
    dims: Rect,

    cursor_blink_on: bool,
    cursor_pos: Point2<u32>,

    block_types: Vec<BlockType>,
    glyphs: Vec<Glyph>,
    did_raster: bool,
}

// @Memory we can probably reduce the number of fields used here. Not
// all of these are necessary
//                          - Albert Liu, Dec 18, 2021 Sat 14:31 EST
#[derive(Debug)]
enum FlowResult {
    NotFound,
    Found {
        index: usize, // textual index
    },

    // Visual line, not textual
    FoundLineBegin {
        begin: usize, // textual index
    },

    // Visual line, not textual
    FoundLine {
        pos: Point2<u32>,
        begin: usize, // textual index
        end: usize,   // textual index
    },
}

impl View {
    pub fn new(dims: Rect, cache: &mut GlyphCache) -> Self {
        let size = (dims.x * dims.y) as usize;
        let mut glyphs = Vec::with_capacity(size);
        let mut block_types = Vec::with_capacity(size);

        for _ in 0..size {
            block_types.push(BlockType::Normal);
            glyphs.push(EMPTY_GLYPH);
        }

        return Self {
            start: 0,
            dims,

            cursor_blink_on: true,
            cursor_pos: Point2 { x: 0, y: 0 },

            block_types,
            glyphs,
            did_raster: true,
        };
    }

    pub fn run<'a>(
        &mut self,
        file: &File,
        command: ViewCommand<'a>,
        output: &mut Vec<TedCommand<'a>>,
    ) {
        match command {
            ViewCommand::CursorMove(direction) => self.cursor_move(direction, output),
            ViewCommand::ToggleCursorBlink => self.toggle_cursor_blink(output),
            ViewCommand::Insert { text } => self.insert(file, text, output),
            ViewCommand::Delete {} => self.delete(file, output),
            ViewCommand::FlowCursor { index } => self.flow_cursor(file, index),
        }
    }

    pub fn draw(&mut self, text: &File, glyphs: &mut GlyphCache) {
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
        self.did_raster = false;
        let atlas_dims = glyphs.atlas_dims();

        let result = TEXT_SHADER.with(|shader| -> Result<(), JsValue> {
            shader.render(
                atlas,
                &self.block_types,
                &self.glyphs,
                atlas_dims,
                self.dims,
            )?;

            return Ok(());
        });

        expect(result);
    }

    fn insert<'a>(&mut self, file: &File, s: &'a str, output: &mut Vec<TedCommand<'a>>) {
        self.cursor_blink_on = true;

        if s.len() == 0 {
            output.push(TedCommand::RequestRedraw);
            return;
        }

        let (flow, result) = self.file_cursor(file);

        let index = match result {
            FlowResult::Found { index } => index,
            FlowResult::FoundLine { pos, begin, end } => {
                let mut index = begin + pos.x as usize;
                if s.chars().nth(0).unwrap() != '\n' {
                    for x in pos.x..self.cursor_pos.x {
                        output.push(TedCommand::InsertText {
                            index: end,
                            text: "~",
                        });

                        index += 1;
                    }
                }

                index
            }
            FlowResult::FoundLineBegin { begin } => {
                let mut index = begin + flow.pos.x as usize;

                if s.chars().nth(0).unwrap() != '\n' {
                    for x in flow.pos.x..self.cursor_pos.x {
                        output.push(TedCommand::AppendText { text: "~" });
                        index += 1;
                    }
                    index = begin + self.cursor_pos.x as usize;
                }

                index
            }
            FlowResult::NotFound => {
                let mut index = flow.index;
                for y in flow.pos.y..self.cursor_pos.y {
                    output.push(TedCommand::AppendText { text: "\n" });
                    index += 1;
                }

                if s.chars().nth(0).unwrap() != '\n' {
                    for x in 0..self.cursor_pos.x {
                        output.push(TedCommand::AppendText { text: "~" });
                        index += 1;
                    }
                }

                index
            }
        };

        let text_index = self.start + index;

        output.push(TedCommand::InsertText {
            index: text_index,
            text: s,
        });

        let count = s.chars().count();
        output.push(for_view(ViewCommand::FlowCursor {
            index: index + count,
        }));

        output.push(TedCommand::RequestRedraw);
    }

    fn delete(&mut self, file: &File, output: &mut Vec<TedCommand>) {
        if file.len() == 0 {
            self.cursor_move(Direction::Left, output);
            return;
        }

        let (flow, result) = self.file_cursor(file);

        let index = match result {
            FlowResult::Found { index } => index,
            _ => {
                self.cursor_move(Direction::Left, output);
                return;
            }
        };
        let text_index = self.start + index;

        self.cursor_blink_on = true;

        if index == 0 {
            output.push(TedCommand::RequestRedraw);
            return;
        }

        output.push(TedCommand::DeleteText {
            begin: text_index - 1,
            end: index,
        });

        output.push(for_view(ViewCommand::FlowCursor { index: index - 1 }));

        output.push(TedCommand::RequestRedraw);
    }

    fn flow_cursor(&mut self, file: &File, index: usize) {
        if file.len() == 0 {
            self.cursor_pos = Point2 { x: 0, y: 0 };
            return;
        }

        // TODO flowing past the end of the screen
        let text = file.text_after_cursor(self.start).unwrap();
        let mut next_pos = None;
        let flow = flow_text(text, self.dims, |state, write_len, c| {
            if state.index == index {
                next_pos = Some(state.pos);
            }
        });

        if flow.index == index && !flow.is_full {
            next_pos = Some(flow.pos);
        }

        if let Some(pos) = next_pos {
            self.cursor_pos = pos;
        }
    }

    fn toggle_cursor_blink(&mut self, output: &mut Vec<TedCommand>) {
        self.cursor_blink_on = !self.cursor_blink_on;
        output.push(TedCommand::RequestRedraw);
    }

    fn cursor_move(&mut self, direction: Direction, output: &mut Vec<TedCommand>) {
        match direction {
            Direction::Up => {
                if self.cursor_pos.y > 0 {
                    self.cursor_pos.y -= 1;
                }
            }
            Direction::Down => {
                if self.cursor_pos.y < self.dims.y - 1 {
                    self.cursor_pos.y += 1;
                }
            }
            Direction::Left => {
                if self.cursor_pos.x > 0 {
                    self.cursor_pos.x -= 1;
                }
            }
            Direction::Right => {
                if self.cursor_pos.x < self.dims.x - 1 {
                    self.cursor_pos.x += 1;
                }
            }
        }

        self.cursor_blink_on = true;
        output.push(TedCommand::RequestRedraw);
    }

    fn file_cursor(&self, file: &File) -> (FlowState, FlowResult) {
        if file.len() == 0 {
            let pos = Point2 { x: 0, y: 0 };
            let result = match self.cursor_pos == pos {
                true => FlowResult::Found { index: 0 },
                false => FlowResult::NotFound,
            };

            let flow = FlowState {
                index: 0,
                is_full: false,
                pos,
            };

            return (flow, result);
        }

        let mut result = FlowResult::NotFound;
        let text = file.text_after_cursor(self.start).unwrap();
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

        return (flow, result);
    }
}

#[derive(Clone, Copy)]
struct FlowState {
    is_full: bool,
    pos: Point2<u32>,
    index: usize,
}

// eventually this only cares about width maybe?
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
