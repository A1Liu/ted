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
pub struct Style {
    pub fg_color: Color,
    pub bg_color: Option<Color>,
}

pub struct Highlighter {
    short_seq: Vec<Rule<char>>,
    exact_seq: Vec<Rule<Vec<char>>>,
}

#[cfg_attr(debug_assertions, derive(PartialEq))]
pub struct RangeData {
    pub offset_from_last: usize,
    pub len: usize,
    pub style: Style,
}

impl Highlighter {
    pub fn new(rules: Vec<SyntaxRule>) -> Self {
        let mut short_seq = Vec::new();
        let mut exact_seq = Vec::new();

        for rule in rules {
            let style = rule.style;
            match rule.pattern {
                Pattern::ExactShort(pattern) => {
                    short_seq.push(Rule { pattern, style });
                }
                Pattern::Exact(pattern) => {
                    let pattern = pattern.chars().collect();
                    exact_seq.push(Rule { pattern, style });
                }
            }
        }

        short_seq.shrink_to_fit();
        exact_seq.shrink_to_fit();

        return Self {
            short_seq,
            exact_seq,
        };
    }

    pub fn ranges(&self, text: &[char]) -> Vec<RangeData> {
        let mut index = 0;
        let mut prev_index = 0;
        let mut data = Vec::new();

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
            let exact_match = exact_iter.find(|r| text[index..].starts_with(&r.pattern));
            if let Some(r) = exact_match {
                let len = r.pattern.len();

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

pub struct Rule<P> {
    pub pattern: P,
    pub style: Style,
}

pub enum Pattern {
    ExactShort(char),
    Exact(String),
}
