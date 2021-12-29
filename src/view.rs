use crate::commands::*;
use crate::graphics::*;
use crate::text::*;
use crate::util::*;
use winit::event;

pub struct View {
    start: usize,
    start_line: usize,
    dims: Rect,

    cursor_blink_on: bool,
    cursor_pos: Point2<u32>,

    visible_text: Vec<char>,
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
    pub fn new(dims: Rect, s: &str) -> Self {
        let size = (dims.x * dims.y) as usize;
        let mut glyphs = Vec::with_capacity(size);
        let mut block_types = Vec::with_capacity(size);
        let mut visible_text = Vec::with_capacity(size);

        for _ in 0..size {
            block_types.push(BlockType::Normal);
            glyphs.push(EMPTY_GLYPH);
        }

        visible_text.extend(s.chars());

        return Self {
            start: 0,
            start_line: 0,
            dims,

            cursor_blink_on: true,
            cursor_pos: Point2 { x: 0, y: 0 },

            visible_text,
            block_types,
            glyphs,
            did_raster: true,
        };
    }

    pub fn run<'a>(&mut self, command: ViewCommand<'a>, output: &mut Vec<TedCommand<'a>>) {
        match command {
            ViewCommand::CursorMove(direction) => self.cursor_move(direction, output),
            ViewCommand::ToggleCursorBlink => self.toggle_cursor_blink(output),
            ViewCommand::Insert { text } => self.insert(text, output),
            ViewCommand::Delete {} => self.delete(output),
            ViewCommand::FlowCursor { index } => self.flow_cursor(index),
            ViewCommand::SetContents { start, text } => self.set_contents(start, text, output),
        }
    }

    pub fn draw(&mut self, glyphs: &mut GlyphCache) {
        let config = FlowConfig {
            text: self.visible_text.iter().map(|c| *c),
            wrap_width: Some(self.dims.x),
            vertical_bound: Some(self.dims.y),
        };

        let mut line_numbers = vec![None; self.dims.y as usize];
        let mut line = self.start_line + 1;
        let mut display_line = Some(line);
        let state = flow_text(config, |state, params| {
            if state.pos.x == 0 {
                line_numbers[state.pos.y as usize] = display_line.take();
            }

            if params.c == '\n' {
                display_line = Some(state.newline_count + line);
            }

            let idx = (state.pos.y * self.dims.x + state.pos.x) as usize;
            let write_len = params.write_len as usize;
            match params.c.is_whitespace() {
                true => self.glyphs[idx..(idx + write_len)].fill(EMPTY_GLYPH),
                false => {
                    let mut tmp = [0; 4];
                    let c_str = params.c.encode_utf8(&mut tmp);
                    let glyph_list = glyphs.translate_glyphs(c_str);
                    self.did_raster = self.did_raster || glyph_list.did_raster;
                    self.glyphs[idx..(idx + write_len)].fill(glyph_list.glyphs[0]);
                }
            }

            if params.will_wrap {
                let begin = idx + params.write_len as usize;
                let end = idx + (self.dims.x - state.pos.x) as usize;
                self.glyphs[begin..end].fill(EMPTY_GLYPH);
            }
        });

        let mut pt = state.pos;

        // Fill remaining glyphs with the empty glyph
        for y in pt.y..self.dims.y {
            if pt.x == 0 {
                line_numbers[y as usize] = display_line.take();
            }

            let len = (self.dims.x - pt.x) as usize;
            let idx = (y * self.dims.x + pt.x) as usize;

            self.glyphs[idx..(idx + len)].fill(EMPTY_GLYPH);
            pt.x = 0;
        }

        assert_eq!(line_numbers.len(), self.dims.y as usize);

        const LINES_WIDTH: usize = 3;
        let (line_block_types, line_glyphs) = {
            let size = LINES_WIDTH * self.dims.y as usize;
            let mut number_glyphs = Vec::with_capacity(size);
            let mut line_block_types = Vec::with_capacity(size);

            for _ in 0..size {
                line_block_types.push(BlockType::Normal);
            }

            let mut write_to;
            let mut line_glyphs;
            for line in &line_numbers {
                write_to = [b' '; LINES_WIDTH];

                if let Some(line) = line {
                    use std::io::Write;

                    let mut buf: &mut [u8] = &mut write_to;
                    write!(buf, "{: >width$}", line, width = LINES_WIDTH).unwrap();
                }

                line_glyphs = write_to.map(|b| {
                    let mut tmp = [0; 4];
                    let c_str = char::from_u32(b as u32).unwrap().encode_utf8(&mut tmp);
                    let glyph_list = glyphs.translate_glyphs(c_str);
                    self.did_raster = self.did_raster || glyph_list.did_raster;

                    return glyph_list.glyphs[0];
                });

                number_glyphs.extend_from_slice(&line_glyphs);
            }

            (line_block_types, number_glyphs)
        };

        let mut atlas = self.did_raster.then(|| glyphs.atlas());
        self.did_raster = false;
        let atlas_dims = glyphs.atlas_dims();

        // render line numbers
        let result = TEXT_SHADER.with(|shader| -> Result<(), JsValue> {
            shader.render(
                true,
                atlas.take(),
                &line_block_types,
                &line_glyphs,
                atlas_dims,
                Rect {
                    x: LINES_WIDTH as u32,
                    y: self.dims.y,
                },
            )?;

            return Ok(());
        });

        expect(result);

        // clear block state
        for block_type in &mut self.block_types {
            *block_type = BlockType::Normal;
        }

        if self.cursor_blink_on {
            let idx = self.cursor_pos.y * self.dims.x + self.cursor_pos.x;
            self.block_types[idx as usize] = BlockType::Cursor;
        }

        let result = TEXT_SHADER.with(|shader| -> Result<(), JsValue> {
            shader.render(
                false,
                atlas,
                &self.block_types,
                &self.glyphs,
                atlas_dims,
                self.dims,
            )?;

            return Ok(());
        });

        let mut block_types = Vec::with_capacity(4 * self.dims.y as usize);
        for _ in 0..(4 * self.dims.y) {
            block_types.push(BlockType::Normal);
        }

        expect(result);
    }

    fn set_contents(&mut self, start: usize, text: &str, output: &mut Vec<TedCommand>) {
        self.start = start;
        self.visible_text.clear();

        // TODO flowing past the end of the screen
        let config = FlowConfig {
            text: text.chars(),
            wrap_width: Some(self.dims.x),
            vertical_bound: Some(self.dims.y),
        };

        flow_text(config, |state, params| {
            self.visible_text.push(params.c);
        });

        output.push(TedCommand::RequestRedraw);
    }

    fn insert<'a>(&mut self, s: &'a str, output: &mut Vec<TedCommand<'a>>) {
        if s.len() == 0 {
            output.push(TedCommand::RequestRedraw);
            return;
        }

        let (flow, result) = self.file_cursor();

        let index = match result {
            FlowResult::Found { index } => index,
            FlowResult::FoundLine { pos, begin, end } => {
                let mut index = begin + pos.x as usize;
                if s.chars().nth(0).unwrap() != '\n' {
                    for x in pos.x..self.cursor_pos.x {
                        self.visible_text.insert(index, '~');
                        index += 1;
                    }
                }

                index
            }
            FlowResult::FoundLineBegin { begin } => {
                let mut index = begin + flow.pos.x as usize;

                if s.chars().nth(0).unwrap() != '\n' {
                    for x in flow.pos.x..self.cursor_pos.x {
                        self.visible_text.push('~');
                        index += 1;
                    }
                    index = begin + self.cursor_pos.x as usize;
                }

                index
            }
            FlowResult::NotFound => {
                let mut index = flow.index;
                for y in flow.pos.y..self.cursor_pos.y {
                    self.visible_text.push('\n');
                    index += 1;
                }

                if s.chars().nth(0).unwrap() != '\n' {
                    for x in 0..self.cursor_pos.x {
                        self.visible_text.push('~');
                        index += 1;
                    }
                }

                index
            }
        };

        let text_index = self.start + index;

        self.visible_text.splice(index..index, s.chars());

        let count = s.chars().count();
        self.flow_cursor(index + count);

        output.push(TedCommand::RequestRedraw);
    }

    fn delete(&mut self, output: &mut Vec<TedCommand>) {
        let (flow, result) = self.file_cursor();

        let index = match result {
            FlowResult::Found { index } => index,
            _ => {
                self.cursor_move(Direction::Left, output);
                return;
            }
        };

        let text_index = self.start + index;
        if text_index == 0 {
            output.push(TedCommand::RequestRedraw);
            return;
        }

        self.visible_text.remove(index - 1);
        self.flow_cursor(index - 1);

        output.push(TedCommand::RequestRedraw);
    }

    fn flow_cursor(&mut self, index: usize) {
        self.cursor_blink_on = true;
        if self.visible_text.len() == 0 {
            self.cursor_pos = Point2 { x: 0, y: 0 };
            return;
        }

        // TODO flowing past the end of the screen
        let config = FlowConfig {
            text: self.visible_text.iter().map(|c| *c),
            wrap_width: Some(self.dims.x),
            vertical_bound: Some(self.dims.y),
        };

        let mut next_pos = None;
        let flow = flow_text(config, |state, params| {
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

    // TODO This should maybe include settings and whatnot
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

    fn file_cursor(&self) -> (FlowState, FlowResult) {
        let config = FlowConfig {
            text: self.visible_text.iter().map(|c| *c),
            wrap_width: Some(self.dims.x),
            vertical_bound: Some(self.dims.y),
        };

        let mut result = FlowResult::NotFound;
        let flow = flow_text(config, |state, params| {
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
                        if params.c == '\n' {
                            let (pos, begin, end) = (state.pos, state.index, state.index);
                            *r = FlowResult::FoundLine { pos, begin, end };
                        }
                    }
                }
                FlowResult::FoundLineBegin { begin } => {
                    if params.c == '\n' {
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

struct FlowConfig<Iter>
where
    Iter: Iterator<Item = char>,
{
    text: Iter,
    wrap_width: Option<u32>,
    vertical_bound: Option<u32>,
}

#[derive(Clone, Copy)]
struct FlowState {
    is_full: bool,
    pos: Point2<u32>,
    index: usize,
    newline_count: usize,
}

struct FlowParams {
    write_len: u32,
    will_wrap: bool,
    c: char,
}

// eventually this only cares about width maybe?
fn flow_text<Iter, F>(config: FlowConfig<Iter>, mut f: F) -> FlowState
where
    Iter: Iterator<Item = char>,
    F: FnMut(FlowState, &mut FlowParams),
{
    // TODO(design): This handles full newline-terminated lines a bit weirdly.
    // To be fair, Vim handles them a little bit weirdly too. Ideally we want
    // full newline terminated lines to only extend to an additional line
    // when absolutely necessary, like when the user wants to append to a full
    // line. Right now, we just always add an extra blank visual line. It looks
    // kinda ugly though. We probably want to do a generalization/flexibility
    // pass on the flow_text procedure altogether, and allow for more of these
    // kinds of decisions to be made by the callee. Maybe transition to state
    // machine while loop kind of deal?

    let mut state = FlowState {
        is_full: false,
        pos: Point2 { x: 0, y: 0 },
        index: 0,
        newline_count: 0,
    };

    for c in config.text {
        let mut will_wrap = false;

        let write_len = match c {
            '\n' => {
                will_wrap = true;
                state.newline_count += 1;

                0
            }
            '\t' => 2,
            c if c.is_control() => {
                state.index += 1;
                continue;
            }
            c => 1, // TODO grapheme stuffs
        };

        if let Some(width) = config.wrap_width {
            if state.pos.x + write_len >= width {
                will_wrap = true;
            }
        }

        let mut params = FlowParams {
            write_len,
            will_wrap,
            c,
        };

        f(state, &mut params);

        state.pos.x += params.write_len;
        if params.will_wrap {
            state.pos.x = 0;
            state.pos.y += 1;
        }

        if let Some(bound) = config.vertical_bound {
            if state.pos.y >= bound {
                state.is_full = true;
            }
        }

        state.index += 1;
        if state.is_full {
            return state;
        }
    }

    return state;
}
