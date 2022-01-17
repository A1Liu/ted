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
pub enum HLAction {
    BeginScope(u32),
    EndScope,
    None,
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

struct RuleAction {
    len: usize,
    style: Style,
    action: HLAction,
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
                let (action, style) = (rule.action, rule.style);

                match &rule.pattern {
                    &Pattern::ExactShort(pattern) => {
                        short_seq.push(Rule {
                            pattern,
                            action,
                            style,
                        });
                    }
                    Pattern::Exact(pattern) => {
                        seq_data.reserve(pattern.len());

                        let start = seq_data.len();
                        for c in pattern.chars() {
                            seq_data.push(c);
                        }

                        let end = seq_data.len();
                        let pattern = CopyRange { start, end };

                        exact_seq.push(Rule {
                            pattern,
                            action,
                            style,
                        });
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

    // returns true if the scope should end
    fn run_rule(&self, state: &mut HighlightState, action: RuleAction) -> bool {
        let len = action.len;
        let style = action.style;
        let action = action.action;

        let start = state.scope.index;
        let range = r(start, start + len);

        state.data.push(RangeData { range, style });

        match action {
            HLAction::None => state.scope.index = range.end,
            HLAction::EndScope => return true,
            HLAction::BeginScope(id) => {
                let index = range.end;
                let info = self.scope_info[id as usize];
                let new_scope = Scope { index, info };

                state.scope_stack.push(state.scope);
                state.scope = new_scope;
            }
        }

        return false;
    }

    pub fn ranges(&self, text: &[char]) -> Pod<RangeData> {
        let scope = Scope {
            index: 0,
            info: self.scope_info[0],
        };

        let mut state = HighlightState {
            scope,
            scope_stack: pod![scope],
            data: Pod::new(),
        };

        let end = text.len();

        while state.scope.index < end {
            state.scope = match state.scope_stack.pop() {
                Some(scope) => scope,
                None => break,
            };

            let short_seq = &self.short_seq[state.scope.info.seqs];
            let exact_seq = &self.exact_seq[state.scope.info.exact_seqs];

            'scope: while state.scope.index < end {
                let index = state.scope.index;

                for rule in short_seq {
                    if rule.pattern != text[index] {
                        continue;
                    }

                    let action = RuleAction {
                        len: 1,
                        action: rule.action,
                        style: rule.style,
                    };

                    match self.run_rule(&mut state, action) {
                        true => break 'scope,
                        false => continue 'scope,
                    }
                }

                for rule in exact_seq {
                    if !text[index..].starts_with(&self.seq_data[rule.pattern]) {
                        continue;
                    }

                    let action = RuleAction {
                        len: rule.pattern.len(),
                        action: rule.action,
                        style: rule.style,
                    };

                    match self.run_rule(&mut state, action) {
                        true => break 'scope,
                        false => continue 'scope,
                    }
                }

                state.scope.index += 1;
            }
        }

        return state.data;
    }
}

pub type SyntaxRule = Rule<Pattern>;

#[derive(Clone, Copy)]
pub struct Rule<P> {
    pub pattern: P,
    pub style: Style,
    pub action: HLAction,
}

pub enum Pattern {
    ExactShort(char),
    Exact(String),
}
