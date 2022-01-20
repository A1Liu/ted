use crate::gon::*;
use crate::types::*;
use crate::util::*;
use mint::*;
use std::collections::hash_map::HashMap;

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
    // Builtins:
    // default scope # the default scope
    //
    // default {
    //   # default rule for scope; for the default scope, it is required, and
    //   # all its fields are also required.
    //   # Non-default scopes fallback to default scope rule if none is provided
    //
    //   color [254 254 254] # RBG color for text
    //   background [0 0 0] # RBG color for background
    //
    //   # Cannot contain a "match" or "scope"
    // }
    //
    // hello {
    //   match >a
    //   scope end # end the scope
    //
    //   color [254 254 254] # optional
    //   background [0 0 0] # optional
    // }
    //
    pub fn from_gon<'a>(gon: &'a str) -> Self {
        let gon = parse_gon(gon);
        let (values, fields) = match gon {
            GonValue::Object { values, fields } => (values, fields),
            _ => panic!("Expected a GON object"),
        };

        #[derive(Clone, Copy)]
        enum ScopeAction<'a> {
            None,
            Scope(&'a str),
            End,
        }

        struct DefaultRule {
            color: Option<Color>,
            background: Option<Color>,
        }

        #[derive(Clone, Copy)]
        struct Rule<'a> {
            pattern: &'a str,
            color: Option<Color>,
            background: Option<Color>,
            scope: ScopeAction<'a>,
        }

        #[derive(Clone, Copy)]
        enum Variable<'a> {
            Rule(Rule<'a>),
            Color(Color),
        }

        struct Scope<'a> {
            default: Option<DefaultRule>,
            rules: Pod<Rule<'a>>,
        }

        fn expect_color_value(g: Option<&GonValue>) -> f32 {
            let g = unwrap(g);

            if let GonValue::Str(s) = g {
                let value = expect(s.parse::<u8>());

                return value as f32;
            }

            panic!("what the hell");
        }

        fn expect_color(g: &GonValue) -> Color {
            if let GonValue::Array(values) = g {
                if values.len() != 3 {
                    panic!("colors have 3 fields (RGB)");
                }

                let r = expect_color_value(values.get(0));
                let g = expect_color_value(values.get(1));
                let b = expect_color_value(values.get(2));

                return color(r, g, b);
            }

            panic!("what the hell");
        }

        fn get_field<'a, 'b>(
            values: &'b Vec<(&'a str, GonValue<'a>)>,
            fields: &HashMap<&'a str, usize>,
            name: &str,
        ) -> Option<&'b GonValue<'a>> {
            let index = fields.get(name)?;
            let (_, value) = unwrap(values.get(*index));
            return Some(value);
        }

        let mut scopes: HashMap<&'a str, Scope<'a>> = HashMap::new();
        let mut variables = HashMap::new();

        let mut scope_name = "default";
        let mut default_rule: Option<DefaultRule> = None;
        let mut rules = Pod::new();
        for (name, value) in values {
            // used to extend the lifetime of GonValue::String values
            let temp;

            let text = match value {
                // Parse as scope's default rule
                GonValue::Object { values, fields } if name == "default" => {
                    if default_rule.is_some() {
                        panic!("already defined default rule for current scope");
                    }

                    let color = get_field(&values, &fields, "color").map(expect_color);
                    let background = get_field(&values, &fields, "background").map(expect_color);

                    default_rule = Some(DefaultRule { color, background });
                    continue;
                }

                // Parse as rule
                GonValue::Object { values, fields } => {
                    let pattern = unwrap(get_field(&values, &fields, "match"));
                    let pattern = match pattern {
                        GonValue::Str(s) => *s,
                        _ => panic!("shoulda been a string"),
                    };

                    let color = get_field(&values, &fields, "color").map(expect_color);
                    let background = get_field(&values, &fields, "background").map(expect_color);

                    let scope = get_field(&values, &fields, "pattern").map(|v| {
                        if let GonValue::Str(s) = v {
                            return *s;
                        }

                        panic!("shoulda been a string");
                    });

                    let scope = match scope {
                        None => ScopeAction::None,
                        Some("end") => ScopeAction::End,
                        Some(name) => ScopeAction::Scope(name),
                    };

                    rules.push(Rule {
                        pattern,
                        color,
                        background,
                        scope,
                    });

                    continue;
                }

                // Parse as color variable
                GonValue::Array(values) => {
                    let color = expect_color(&GonValue::Array(values));

                    if let Some(prev) = variables.insert(name, color) {
                        panic!("variable redefined");
                    }

                    continue;
                }

                GonValue::Str(s) => s,
                GonValue::String(s) => {
                    temp = s;
                    &temp
                }
            };

            if text == "scope" {
                let scope = scopes.entry(scope_name).or_insert(Scope {
                    default: None,
                    rules: Pod::new(),
                });

                if let Some(rule) = default_rule.take() {
                    if scope.default.is_some() {
                        panic!("default rule already defined for scope");
                    }

                    scope.default = Some(rule);
                }

                scope.rules.reserve(rules.len());
                for &rule in rules.iter() {
                    scope.rules.push(rule);
                }

                rules.clear();

                scope_name = name;
                continue;
            }

            // TODO other stuffs, idk
            panic!("expected 'scope' declaration");
        }

        let scope = scopes.entry(scope_name).or_insert(Scope {
            default: None,
            rules: Pod::new(),
        });

        if let Some(rule) = default_rule.take() {
            if scope.default.is_some() {
                panic!("default rule already defined for scope");
            }

            scope.default = Some(rule);
        }

        scope.rules.reserve(rules.len());
        for &rule in rules.iter() {
            scope.rules.push(rule);
        }

        rules.clear();

        let mut rules = Vec::new();
        let mut scope_ranges = Pod::new();

        return Self::new(rules, Some(scope_ranges));
    }

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

            for rule in unwrap(rules.get(scope.start..scope.end)) {
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
