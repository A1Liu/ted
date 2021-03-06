use crate::editor::highlighting::*;
use crate::editor::text::*;
use crate::editor::types::*;
use crate::util::*;
use std::io::Write;

pub struct View {
    start: usize,
    start_line: usize,
    dims: Rect,

    cursor_blink_on: bool,
    cursor_pos: Point2<u32>,

    visible_text: Pod<char>,
    highlighter: Highlighter,
}

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
        let mut visible_text = Pod::with_capacity(size);

        let mut config = FlowConfig::new(s.chars(), Some(dims.x), Some(dims.y));
        for (state, params) in &mut config {
            visible_text.push(params.c);
        }

        let highlighter = Highlighter::from_gon(include_str!("../test_highlighter.gon"));

        return Self {
            start: 0,
            start_line: 0,
            dims,

            cursor_blink_on: true,
            cursor_pos: Point2 { x: 0, y: 0 },

            visible_text,
            highlighter,
        };
    }

    pub fn run(&mut self, command: Command<ViewCommand>) {
        let output = command.buffer;

        match command.value {
            ViewCommand::CursorMove(direction) => self.cursor_move(direction, output),
            ViewCommand::ToggleCursorBlink => self.toggle_cursor_blink(output),
            ViewCommand::Insert { text } => self.insert(text, output),
            ViewCommand::DeleteAfterCursor => self.delete(output),
            ViewCommand::FlowCursor { index } => self.flow_cursor(index),
            ViewCommand::SetContents(contents) => self.set_contents(contents, output),
            ViewCommand::Draw => self.draw(output),
        }
    }

    pub fn draw(&self, output: &mut Vec<TedCommand>) {
        let mut text_fg_colors = pod![DEFAULT_FG; self.visible_text.len()];
        let mut text_bg_colors = pod![DEFAULT_BG; self.visible_text.len()];

        for range in self.highlighter.ranges(&self.visible_text) {
            text_fg_colors[range.range].fill(range.color);
            text_bg_colors[range.range].fill(range.background);
        }

        let mut config = FlowConfig::new(self.chars(), Some(self.dims.x), Some(self.dims.y));

        let size = (self.dims.x * self.dims.y) as usize;
        let mut text = pod![' '; size];
        let mut fg_colors = pod![DEFAULT_FG; size];
        let mut bg_colors = pod![DEFAULT_BG; size];
        let mut line_numbers = pod![None; self.dims.y as usize];

        let line = self.start_line + 1;
        let mut display_line = Some(line);

        for (state, params) in &mut config {
            if state.pos.x == 0 {
                line_numbers[state.pos.y as usize] = display_line.take();
            }

            let begin = (state.pos.y * self.dims.x + state.pos.x) as usize;
            let end = begin + params.write_len as usize;
            fg_colors[begin..end].fill(text_fg_colors[state.index]);
            bg_colors[begin..end].fill(text_bg_colors[state.index]);

            match params.c {
                '\n' => {
                    display_line.replace(state.newline_count + line);
                }

                c if c.is_whitespace() => {}

                c => {
                    text[begin..end].fill(c);
                }
            }
        }

        // Fill remaining glyphs with the empty glyph
        let state = config.finalize();
        if state.pos.y < self.dims.y && state.pos.x == 0 {
            line_numbers[state.pos.y as usize] = display_line.take();
        }

        debug_assert_eq!(line_numbers.len(), self.dims.y as usize);

        // clear block state
        if self.cursor_blink_on {
            let idx = (self.cursor_pos.y * self.dims.x + self.cursor_pos.x) as usize;
            fg_colors[idx] = DEFAULT_BG;
            bg_colors[idx] = color(1.0, 1.0, 1.0);
        }

        output.push(TedCommand::DrawView {
            is_lines: false,
            fg_colors,
            bg_colors,
            text,
            dims: self.dims,
        });

        const LINES_WIDTH: usize = 3;
        let line_size = LINES_WIDTH * self.dims.y as usize;
        let mut line_text = Pod::with_capacity(line_size);

        for line in line_numbers {
            let mut write_to = [b' '; LINES_WIDTH];
            if let Some(line) = line {
                let mut buf: &mut [u8] = &mut write_to;
                expect(write!(buf, "{: >width$}", line, width = LINES_WIDTH));
            }

            for b in write_to {
                let c = unwrap(char::from_u32(b as u32));
                line_text.push(c);
            }
        }

        let mut fg_colors = pod![LINES_FG; line_size];
        let mut bg_colors = pod![LINES_BG; line_size];

        output.push(TedCommand::DrawView {
            is_lines: true,
            fg_colors,
            bg_colors,
            text: line_text,
            dims: new_rect(LINES_WIDTH as u32, self.dims.y),
        });
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

    fn insert(&mut self, s: String, output: &mut Vec<TedCommand>) {
        let first_char = match s.chars().next() {
            Some(c) => c,
            None => {
                output.push(TedCommand::RequestRedraw);
                return;
            }
        };

        let (flow, result) = self.file_cursor();

        let index = match result {
            FlowResult::Found { index } => index,

            FlowResult::FoundLine { end_pos, begin } => {
                let mut index = begin + end_pos.x as usize;

                if first_char != '\n' {
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

                if first_char != '\n' {
                    for x in 0..self.cursor_pos.x {
                        self.visible_text.push('~');
                        index += 1;
                    }
                }

                index
            }
        };

        let mut chars = Pod::with_capacity(s.len());
        for c in s.chars() {
            chars.push(c);
        }

        let count = chars.len();

        self.visible_text.splice(index..index, &chars);

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
        let mut config = FlowConfig::new(self.chars(), Some(self.dims.x), Some(self.dims.y));

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
        let mut config = FlowConfig::new(self.chars(), Some(self.dims.x), Some(self.dims.y));

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

    fn chars<'a>(&'a self) -> impl Iterator<Item = char> + 'a {
        // Previously attempted to replace this code with a handmade, non-generic
        // version. Ended up growing the binary from 237.69kb to 237.71kb.
        //                              - Albert Liu, Jan 10, 2022 Mon 01:06 EST
        return self.visible_text.iter().map(|c| *c);
    }
}
