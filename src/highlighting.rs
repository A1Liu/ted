use crate::util::*;
use mint::*;

pub type Color = Vector3<f32>;

pub const fn color(r: f32, g: f32, b: f32) -> Color {
    return Color { x: r, y: g, z: b };
}

pub const NORMAL: Color = color(0.933, 0.91, 0.835);
pub const TEXT_BG: Color = color(0.0, 0.169, 0.212);
pub const KEYWORD: Color = color(0.522, 0.6, 0.0);

pub const LINES_FG: Color = color(0.396, 0.482, 0.514);
pub const LINES_BG: Color = color(0.027, 0.212, 0.259);

pub const DEFAULT_FG: Color = NORMAL;
pub const DEFAULT_BG: Color = TEXT_BG;

#[derive(Clone, Copy)]
#[cfg_attr(debug_assertions, derive(PartialEq))]
pub struct CopyRange {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy)]
#[cfg_attr(debug_assertions, derive(PartialEq))]
pub struct Style {
    pub fg_color: Color,
    pub bg_color: Option<Color>,
}

pub struct Highlighter {
    seq_data: Pod<char>,
    short_seq: Pod<Rule<char>>,
    exact_seq: Pod<Rule<CopyRange>>,
}

#[derive(Clone, Copy)]
#[cfg_attr(debug_assertions, derive(PartialEq))]
pub struct RangeData {
    pub offset_from_last: usize,
    pub len: usize,
    pub style: Style,
}

impl Highlighter {
    pub fn new(rules: Vec<SyntaxRule>) -> Self {
        let mut seq_data = Pod::new();
        let mut short_seq = Pod::new();
        let mut exact_seq = Pod::new();

        for rule in rules {
            let style = rule.style;
            match rule.pattern {
                Pattern::ExactShort(pattern) => {
                    short_seq.push(Rule { pattern, style });
                }
                Pattern::Exact(pattern) => {
                    seq_data.reserve(pattern.len());

                    let start = seq_data.len();
                    for c in pattern.chars() {
                        seq_data.push(c);
                    }

                    let end = seq_data.len();
                    let pattern = CopyRange { start, end };

                    exact_seq.push(Rule { pattern, style });
                }
            }
        }

        seq_data.shrink_to_fit();
        short_seq.shrink_to_fit();
        exact_seq.shrink_to_fit();

        return Self {
            seq_data,
            short_seq,
            exact_seq,
        };
    }

    pub fn ranges(&self, text: &[char]) -> Pod<RangeData> {
        let mut index = 0;
        let mut prev_index = 0;
        let mut data = Pod::new();

        while index < text.len() {
            if let Some(r) = self.short_seq.iter().find(|r| r.pattern == text[index]) {
                data.push(RangeData {
                    offset_from_last: index - prev_index,
                    len: 1,
                    style: r.style,
                });

                index += 1;
                prev_index = index;
                continue;
            }

            let mut exact_iter = self.exact_seq.iter();
            let exact_match = exact_iter.find(|r| {
                let (start, end) = (r.pattern.start, r.pattern.end);
                return text[index..].starts_with(&self.seq_data[start..end]);
            });
            if let Some(r) = exact_match {
                let len = r.pattern.end - r.pattern.start;

                data.push(RangeData {
                    offset_from_last: index - prev_index,
                    len,
                    style: r.style,
                });

                index += len;
                prev_index = index;
                continue;
            }

            index += 1;
        }

        return data;
    }
}

pub type SyntaxRule = Rule<Pattern>;

#[derive(Clone, Copy)]
pub struct Rule<P> {
    pub pattern: P,
    pub style: Style,
}

pub enum Pattern {
    ExactShort(char),
    Exact(String),
}
