use crate::util::*;

// TODO clean this stuff up
pub struct FlowConfig<Iter>
where
    Iter: Iterator<Item = char>,
{
    text: Iter,
    state: FlowState,
    params: FlowParams,
    needs_final: bool,

    // Is this flexibility necessary?
    //                  - Albert Liu, Jan 10, 2022 Mon 01:08 EST
    wrap_width: Option<u32>,
    vertical_bound: Option<u32>,
}

#[derive(Clone, Copy)]
pub struct FlowState {
    pub is_full: bool,
    pub pos: Point2<u32>,
    pub index: usize,
    pub newline_count: usize,
}

impl Default for FlowState {
    fn default() -> Self {
        return Self {
            is_full: false,
            pos: Point2 { x: 0, y: 0 },
            index: 0,
            newline_count: 0,
        };
    }
}

#[derive(Clone, Copy)]
pub struct FlowParams {
    pub write_len: u32,
    pub will_wrap: bool,
    pub c: char,
}

impl<Iter> FlowConfig<Iter>
where
    Iter: Iterator<Item = char>,
{
    pub fn new(text: Iter, wrap_width: Option<u32>, vertical_bound: Option<u32>) -> Self {
        return Self {
            text,
            state: Default::default(),
            params: FlowParams {
                write_len: 0,
                will_wrap: false,
                c: ' ',
            },
            needs_final: false,
            wrap_width,
            vertical_bound,
        };
    }

    pub fn finalize(mut self) -> FlowState {
        self.complete_params();

        return self.state;
    }

    fn complete_params(&mut self) {
        if !self.needs_final {
            return;
        }

        self.needs_final = false;

        self.state.pos.x += self.params.write_len;
        if self.params.will_wrap {
            self.state.pos.x = 0;
            self.state.pos.y += 1;
        }

        if let Some(bound) = self.vertical_bound {
            if self.state.pos.y >= bound {
                self.state.is_full = true;
            }
        }

        self.state.index += 1;
    }
}

// TODO(design): This handles full newline-terminated lines a bit weirdly.
// To be fair, Vim handles them a little bit weirdly too. Ideally we want
// full newline terminated lines to only extend to an additional line
// when absolutely necessary, like when the user wants to append to a full
// line. Right now, we just always add an extra blank visual line. It looks
// kinda ugly though. We probably want to do a generalization/flexibility
// pass on the flow_text procedure altogether, and allow for more of these
// kinds of decisions to be made by the callee. Maybe transition to state
// machine while loop kind of deal?
impl<'a, Iter> Iterator for &'a mut FlowConfig<Iter>
where
    Iter: Iterator<Item = char>,
{
    type Item = (FlowState, FlowParams);

    fn next(&mut self) -> Option<Self::Item> {
        self.complete_params();

        while let Some(c) = self.text.next() {
            if self.state.is_full {
                return None;
            }

            self.params.will_wrap = false;
            self.params.write_len = match c {
                '\n' => {
                    self.params.will_wrap = true;
                    self.state.newline_count += 1;

                    0
                }
                '\t' => 2,
                c if c.is_control() => {
                    self.state.index += 1;
                    continue;
                }
                c => 1, // TODO grapheme stuffs
            };
            self.params.c = c;

            if let Some(width) = self.wrap_width {
                if self.state.pos.x + self.params.write_len >= width {
                    self.params.will_wrap = true;
                }
            }

            self.needs_final = true;
            return Some((self.state, self.params));
        }

        return None;
    }
}
