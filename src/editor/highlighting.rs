use crate::editor::gon::*;
use crate::editor::types::*;
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

#[derive(Debug, Clone, Copy)]
enum RegexItem {
    Exact(char),
}

#[derive(Clone, Copy)]
struct Scope {
    rules: CopyRange<u32>,
    color: Color,
    background: Color,
}

#[derive(Clone, Copy)]
struct Rule {
    pattern: CopyRange<u32>,
    color: Color,
    background: Color,
    action: HLAction,
}

#[derive(Debug, Clone, Copy)]
pub struct HLRange {
    pub range: CopyRange<u32>,
    pub color: Color,
    pub background: Color,
}

pub struct Highlighter {
    regexes: Pod<RegexItem>,
    rules: Pod<Rule>,
    scopes: Pod<Scope>,
}

impl Highlighter {
    pub fn from_gon<'a>(gon: &'a str) -> Self {
        let gon = parse_gon(gon);
        let (mut values, fields) = match gon {
            GonValue::Object { values, fields } => (values, fields),
            _ => panic!("Expected a GON object"),
        };

        #[derive(Clone, Copy)]
        struct IRule {
            pattern: CopyRange<u32>,
            color: Option<Color>,
            background: Option<Color>,
            action: HLAction,
        }

        struct IScope {
            id: usize,
            color: Option<Color>,
            background: Option<Color>,
            rules: Pod<IRule>,
        }

        let mut scopes: HashMap<&'a str, IScope> = HashMap::new();

        scopes.insert(
            "default",
            IScope {
                id: 0,
                color: None,
                background: None,
                rules: Pod::new(),
            },
        );

        for (name, value) in &values {
            let text = match value {
                GonValue::Str(s) => *s,
                GonValue::String(s) => s,

                _ => continue,
            };

            if text == "scope" {
                let id = scopes.len();

                scopes.entry(name).or_insert(IScope {
                    id,
                    color: None,
                    background: None,
                    rules: Pod::new(),
                });
            }
        }

        let mut variables = HashMap::new();
        let mut regexes = Pod::new();
        let mut scope_name = "default";

        for (name, value) in values {
            // used to extend the lifetime of GonValue::String values
            let temp;

            let text = match value {
                // Parse as scope's default rule
                GonValue::Object { values, fields } if name == "default" => {
                    let scope = unwrap(scopes.get_mut(scope_name));

                    let color =
                        get_field(&values, &fields, "color").map(|g| expect_color(&variables, g));
                    let background = get_field(&values, &fields, "background")
                        .map(|g| expect_color(&variables, g));

                    if let Some(prev) = core::mem::replace(&mut scope.color, color) {
                        panic!("already defined default rule's color for current scope");
                    }

                    if let Some(prev) = core::mem::replace(&mut scope.background, background) {
                        panic!("already defined default rule's background for current scope");
                    }

                    continue;
                }

                // Parse as rule
                GonValue::Object { values, fields } => {
                    let pattern = unwrap(get_field(&values, &fields, "match"));
                    let pattern = match pattern {
                        GonValue::Str(s) => *s,
                        _ => panic!("shoulda been a string"),
                    };

                    let begin = regexes.len() as u32;

                    for c in pattern.chars() {
                        regexes.push(RegexItem::Exact(c));
                    }

                    let pattern = r(begin, regexes.len() as u32);

                    let color =
                        get_field(&values, &fields, "color").map(|g| expect_color(&variables, g));
                    let background = get_field(&values, &fields, "background")
                        .map(|g| expect_color(&variables, g));

                    let scope = get_field(&values, &fields, "pattern").map(|v| {
                        if let GonValue::Str(s) = v {
                            return *s;
                        }

                        panic!("shoulda been a string");
                    });

                    let scope_action = match scope {
                        None => HLAction::None,
                        Some("end") => HLAction::EndScope,
                        Some(name) => match scopes.get(name) {
                            Some(scope) => HLAction::BeginScope(scope.id),
                            None => panic!("scope does not exist"),
                        },
                    };

                    let scope = unwrap(scopes.get_mut(scope_name));
                    scope.rules.push(IRule {
                        pattern,
                        color,
                        background,
                        action: scope_action,
                    });

                    continue;
                }

                // Parse as color variable
                GonValue::Array(values) => {
                    let color = expect_color(&variables, &GonValue::Array(values));
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
                scope_name = name;
                continue;
            }

            let color = expect_color(&variables, &GonValue::Str(text));
            if let Some(prev) = variables.insert(name, color) {
                panic!("variable redefined");
            }
        }

        let mut rules = Pod::new();

        let default_scope = unwrap(scopes.get("default"));
        let scope = Scope {
            rules: r(0, 0),
            color: unwrap(default_scope.color),
            background: unwrap(default_scope.background),
        };
        let mut scope_values = pod![scope; scopes.len()];

        for scope in scopes.values() {
            let scope_value = &mut scope_values[scope.id];

            if let Some(color) = scope.color {
                scope_value.color = color;
            }

            if let Some(background) = scope.background {
                scope_value.background = background;
            }

            let start = rules.len() as u32;
            rules.reserve(scope.rules.len());

            for &rule in scope.rules.iter() {
                rules.push(Rule {
                    pattern: rule.pattern,
                    action: rule.action,
                    color: rule.color.unwrap_or(scope_value.color),
                    background: rule.background.unwrap_or(scope_value.background),
                });
            }

            scope_value.rules = r(start, rules.len() as u32);
        }

        return Self {
            regexes,
            rules,
            scopes: scope_values,
        };
    }

    pub fn ranges(&self, text: &[char]) -> Pod<HLRange> {
        let mut ranges = Pod::new();

        // let mut scope_stack = Pod::new();
        let mut scope = self.scopes[0u32];
        let rules = &self.rules[scope.rules];
        let mut current_range = HLRange {
            range: r(0, 0),
            color: scope.color,
            background: scope.background,
        };

        let mut i = 0u32;
        let text_len = text.len() as u32;

        while i < text_len {
            let mut color = scope.color;
            let mut background = scope.background;
            let mut match_range = r(i, i + 1);

            for &rule in rules {
                let pattern = &self.regexes[rule.pattern];

                let (matched, len) = regex_match(pattern, &text[(i as usize)..]);

                if matched {
                    match_range.end = i + len;
                    color = rule.color;
                    background = rule.background;

                    break;
                }
            }

            i = match_range.end;

            if current_range.color == color && current_range.background == background {
                current_range.range.end = match_range.end;
                continue;
            }

            if current_range.range.len() != 0 {
                ranges.push(current_range);
            }

            current_range = HLRange {
                range: match_range,
                color,
                background,
            };
        }

        if current_range.range.len() != 0 {
            ranges.push(current_range);
        }

        return ranges;
    }
}

fn regex_match(regex: &[RegexItem], text: &[char]) -> (bool, u32) {
    let mut regex_index = 0;
    let mut i = 0;
    let len = text.len();

    while i < len && regex_index < regex.len() {
        match regex[regex_index] {
            RegexItem::Exact(c) => {
                if c != text[i] {
                    return (false, 0);
                }

                i += 1;
                regex_index += 1;
            }
        }
    }

    if regex_index < regex.len() {
        return (false, 0);
    }

    return (true, i as u32);
}

fn expect_color_value(g: Option<&GonValue>) -> f32 {
    let g = unwrap(g);

    if let GonValue::Str(s) = g {
        if let Ok(value) = s.parse::<u8>() {
            return (value as f32) / 256.0;
        }

        if let Ok(value) = s.parse::<f32>() {
            return (value as f32) / 256.0;
        }
    }

    panic!("what the hell");
}

fn expect_color<'a>(variables: &HashMap<&'a str, Color>, g: &GonValue<'a>) -> Color {
    match g {
        GonValue::Array(values) => {
            if values.len() != 3 {
                panic!("colors have 3 fields (RGB)");
            }

            let r = expect_color_value(values.get(0));
            let g = expect_color_value(values.get(1));
            let b = expect_color_value(values.get(2));

            return color(r, g, b);
        }

        GonValue::Str(s) => return *unwrap(variables.get(*s)),
        GonValue::String(s) => return *unwrap(variables.get(s.as_str())),

        _ => {}
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
