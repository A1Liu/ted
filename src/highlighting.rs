use crate::types::*;
use crate::util::*;
use mint::*;

pub const NORMAL: Color = color(0.933, 0.91, 0.835);
pub const TEXT_BG: Color = color(0.0, 0.169, 0.212);
pub const KEYWORD: Color = color(0.522, 0.6, 0.0);

pub const LINES_FG: Color = color(0.396, 0.482, 0.514);
pub const LINES_BG: Color = color(0.027, 0.212, 0.259);

pub const DEFAULT_FG: Color = NORMAL;
pub const DEFAULT_BG: Color = TEXT_BG;

const DEFAULT_SCOPE: u32 = 0;

#[derive(Clone, Copy)]
pub enum HighlightAction {
    BeginScope { id: u32, bounded: bool },
    EndScope,
    Style(Style),
}

pub struct Highlighter {
    seq_data: Pod<char>,
    short_seq: Pod<Rule<char>>,
    exact_seq: Pod<Rule<CopyRange>>,
    scope_info: Pod<ScopeInfo>,
}

#[derive(Clone, Copy)]
struct Scope {
    index: usize,
    end: usize,
    info: ScopeInfo,
}

#[derive(Clone, Copy)]
struct ScopeInfo {
    seqs: CopyRange,
    exact_seqs: CopyRange,
}

struct HighlightState {
    scope_stack: Pod<Scope>,
    scope: Scope,
    data: Pod<RangeData>,
}

impl Highlighter {
    pub fn new(rules: Vec<SyntaxRule>, scopes: Option<Pod<CopyRange>>) -> Self {
        let mut seq_data = Pod::new();
        let mut short_seq = Pod::new();
        let mut exact_seq = Pod::new();
        let mut scope_info = Pod::new();

        let default_range = CopyRange {
            start: 0,
            end: rules.len(),
        };

        let scopes = scopes.unwrap_or(pod![default_range]);

        for scope in scopes {
            let seqs_start = short_seq.len();
            let exact_seqs_start = exact_seq.len();

            for rule in &rules[scope.start..scope.end] {
                let action = rule.action;

                match &rule.pattern {
                    &Pattern::ExactShort(pattern) => {
                        short_seq.push(Rule { pattern, action });
                    }
                    Pattern::Exact(pattern) => {
                        seq_data.reserve(pattern.len());

                        let start = seq_data.len();
                        for c in pattern.chars() {
                            seq_data.push(c);
                        }

                        let end = seq_data.len();
                        let pattern = CopyRange { start, end };

                        exact_seq.push(Rule { pattern, action });
                    }
                }
            }

            let scope_data = ScopeInfo {
                seqs: CopyRange {
                    start: seqs_start,
                    end: short_seq.len(),
                },
                exact_seqs: CopyRange {
                    start: exact_seqs_start,
                    end: exact_seq.len(),
                },
            };

            scope_info.push(scope_data);
        }

        seq_data.shrink_to_fit();
        short_seq.shrink_to_fit();
        exact_seq.shrink_to_fit();
        scope_info.shrink_to_fit();

        return Self {
            seq_data,
            short_seq,
            exact_seq,
            scope_info,
        };
    }

    fn run_action(&self, state: &mut HighlightState, len: usize, action: HighlightAction) {
        let start = state.scope.index;
        let end = start + len;

        match action {
            HighlightAction::BeginScope { id, bounded } => {
                let index = if bounded { start } else { end };
                let end = if bounded { end } else { state.scope.end };
                let info = self.scope_info[id as usize];
                let new_scope = Scope { index, end, info };

                state.scope_stack.push(state.scope);
                state.scope = new_scope;
            }

            HighlightAction::EndScope => {
                let prev_scope = unwrap(state.scope_stack.pop());
                state.scope = prev_scope;
            }

            HighlightAction::Style(style) => {
                state.data.push(RangeData { start, end, style });

                state.scope.index = end;
            }
        }
    }

    pub fn ranges(&self, text: &[char]) -> Pod<RangeData> {
        let scope = Scope {
            index: 0,
            end: text.len(),
            info: self.scope_info[0],
        };

        let mut state = HighlightState {
            scope,
            scope_stack: Pod::new(),
            data: Pod::new(),
        };

        while state.scope.index < state.scope.end {
            let short_seq = &self.short_seq[state.scope.info.seqs];
            let exact_seq = &self.exact_seq[state.scope.info.exact_seqs];

            while state.scope.index < state.scope.end {
                let index = state.scope.index;

                if let Some(r) = short_seq.iter().find(|r| r.pattern == text[index]) {
                    self.run_action(&mut state, 1, r.action);
                    continue;
                }

                let mut exact_iter = exact_seq.iter();
                let exact_match = exact_iter.find(|r| {
                    let (start, end) = (r.pattern.start, r.pattern.end);
                    return text[index..].starts_with(&self.seq_data[start..end]);
                });

                if let Some(r) = exact_match {
                    let len = r.pattern.end - r.pattern.start;
                    self.run_action(&mut state, len, r.action);
                    continue;
                }

                state.scope.index += 1;
            }

            match state.scope_stack.pop() {
                Some(scope) => state.scope = scope,
                None => continue,
            }
        }

        return state.data;
    }
}

pub type SyntaxRule = Rule<Pattern>;

#[derive(Clone, Copy)]
pub struct Rule<P> {
    pub pattern: P,
    pub action: HighlightAction,
}

pub enum Pattern {
    ExactShort(char),
    Exact(String),
}
