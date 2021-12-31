use crate::commands::*;
use crate::flow::*;
use crate::graphics::*;
use crate::text::*;
use crate::util::*;
use winit::event;

#[cfg_attr(debug_assertions, derive(PartialEq))]
pub struct View {
    start: usize,
    start_line: usize,
    dims: Rect,

    cursor_blink_on: bool,
    cursor_pos: Point2<u32>,

    visible_text: Vec<char>,
    block_types: Vec<BlockType>,
    glyphs: Vec<Glyph>,
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
    FoundLine {
        end_pos: Point2<u32>,
        begin: usize, // textual index
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

        let mut config = FlowConfig::new(s.chars(), Some(dims.x), Some(dims.y));
        for (state, params) in &mut config {
            visible_text.push(params.c);
        }

        return Self {
            start: 0,
            start_line: 0,
            dims,

            cursor_blink_on: true,
            cursor_pos: Point2 { x: 0, y: 0 },

            visible_text,
            block_types,
            glyphs,
        };
    }

    pub fn run<'a>(&mut self, command: ViewCommand<'a>, output: &mut Vec<TedCommand<'a>>) {
        match command {
            ViewCommand::CursorMove(direction) => self.cursor_move(direction, output),
            ViewCommand::ToggleCursorBlink => self.toggle_cursor_blink(output),
            ViewCommand::Insert { text } => self.insert(text, output),
            ViewCommand::DeleteAfterCursor => self.delete(output),
            ViewCommand::FlowCursor { index } => self.flow_cursor(index),
            ViewCommand::SetContents(contents) => self.set_contents(contents, output),
        }
    }

    // TODO does this need to be more flexible? Do we want to support the terminal
    // target sometime in the future?
    pub fn draw(&mut self, glyphs: &mut GlyphCache) {
        let mut config = FlowConfig::new(
            self.visible_text.iter().map(|c| *c),
            Some(self.dims.x),
            Some(self.dims.y),
        );

        let mut did_raster = false;

        let mut line_numbers = vec![None; self.dims.y as usize];
        let mut line = self.start_line + 1;
        let mut display_line = Some(line);

        for (state, params) in &mut config {
            if state.pos.x == 0 {
                line_numbers[state.pos.y as usize] = display_line.take();
            }

            if params.c == '\n' {
                display_line = Some(state.newline_count + line);
            }

            let row_begin = state.pos.y * self.dims.x;
            let begin = (row_begin + state.pos.x) as usize;
            let end = begin + params.write_len as usize;
            match params.c.is_whitespace() {
                true => self.glyphs[begin..end].fill(EMPTY_GLYPH),
                false => {
                    let res = glyphs.translate_glyph(params.c);
                    did_raster = did_raster || res.did_raster;
                    self.glyphs[begin..end].fill(res.glyph);
                }
            }

            if params.will_wrap {
                let row_end = (row_begin + self.dims.x) as usize;
                self.glyphs[end..row_end].fill(EMPTY_GLYPH);
            }
        }

        // Fill remaining glyphs with the empty glyph
        let state = config.finalize();
        let mut x = state.pos.x;
        for y in state.pos.y..self.dims.y {
            if x == 0 {
                line_numbers[y as usize] = display_line.take();
            }

            let row_begin = y * self.dims.x;
            let begin = (row_begin + x) as usize;
            let end = (row_begin + self.dims.x) as usize;
            self.glyphs[begin..end].fill(EMPTY_GLYPH);
            x = 0;
        }

        debug_assert_eq!(line_numbers.len(), self.dims.y as usize);

        const LINES_WIDTH: usize = 3;
        let (line_block_types, line_glyphs) = {
            let size = LINES_WIDTH * self.dims.y as usize;
            let mut number_glyphs = Vec::with_capacity(size);
            let mut line_block_types = vec![BlockType::Normal; size];

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
                    let c = char::from_u32(b as u32).unwrap();
                    let res = glyphs.translate_glyph(c);
                    did_raster = did_raster || res.did_raster;

                    return res.glyph;
                });

                number_glyphs.extend_from_slice(&line_glyphs);
            }

            (line_block_types, number_glyphs)
        };

        let mut atlas = did_raster.then(|| glyphs.atlas());
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

        expect(result);
    }

    fn set_contents(&mut self, contents: SetContents, output: &mut Vec<TedCommand>) {
        self.start = contents.start;
        self.start_line = contents.start_line;
        self.visible_text.clear();

        let mut config =
            FlowConfig::new(contents.text.chars(), Some(self.dims.x), Some(self.dims.y));
        for (state, params) in &mut config {
            self.visible_text.push(params.c);
        }

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
            FlowResult::FoundLine { end_pos, begin } => {
                let mut index = begin + end_pos.x as usize;

                if s.chars().nth(0).unwrap() != '\n' {
                    for x in end_pos.x..self.cursor_pos.x {
                        self.visible_text.insert(index, '~');
                        index += 1;
                    }
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
        let mut config = FlowConfig::new(
            self.visible_text.iter().map(|c| *c),
            Some(self.dims.x),
            Some(self.dims.y),
        );

        let mut next_pos = None;
        for (state, params) in &mut config {
            if state.index == index {
                next_pos = Some(state.pos);
            }
        }

        let flow = config.finalize();
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
        let mut config = FlowConfig::new(
            self.visible_text.iter().map(|c| *c),
            Some(self.dims.x),
            Some(self.dims.y),
        );

        let mut found_line_end = false;
        let mut result = FlowResult::NotFound;
        for (state, params) in &mut config {
            if state.pos == self.cursor_pos {
                result = FlowResult::Found { index: state.index };
                continue;
            }

            match &mut result {
                FlowResult::Found { .. } => continue,
                FlowResult::FoundLine { end_pos, begin } => {
                    if found_line_end {
                        continue;
                    }

                    *end_pos = state.pos;

                    if params.c == '\n' {
                        found_line_end = true;
                    }
                }
                FlowResult::NotFound => {
                    if state.pos.y == self.cursor_pos.y {
                        let (end_pos, begin) = (state.pos, state.index);
                        result = FlowResult::FoundLine { end_pos, begin };

                        if params.c == '\n' {
                            found_line_end = true;
                        }
                    }
                }
            }
        }

        let flow = config.finalize();
        if flow.pos == self.cursor_pos {
            result = FlowResult::Found { index: flow.index };
        }

        if let FlowResult::FoundLine { end_pos, begin } = &mut result {
            if !found_line_end {
                *end_pos = flow.pos;
            }
        }

        return (flow, result);
    }
}
