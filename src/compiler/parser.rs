use crate::compiler::types::*;
use crate::util::*;
use std::collections::hash_map::HashMap;

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum Keyword {
    Let = 0,
    Proc,
    Type,
    Defer,
    Context,

    If,
    Else,
    Match,

    Continue,
    Break,
    For,

    Spawn,
    Wait,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum TokenKind {
    LParen = b'(',
    RParen = b')',
    LBracket = b'[',
    RBracket = b']',
    LBrace = b'{',
    RBrace = b'}',

    Dot = b'.',
    Comma = b',',
    Colon = b':',
    Semicolon = b';',

    Bang = b'!',
    Tilde = b'~',
    Amp = b'&',
    Caret = b'^',
    Mod = b'%',
    Star = b'*',
    Div = b'/',
    Plus = b'+',
    Dash = b'-',
    Equal = b'=',
    Lt = b'<',
    Gt = b'>',

    Equal2 = 129, // ==
    NotEqual,     // !=
    LtEq,         // <=
    GtEq,         // >=

    And, // &&
    Or,  // ||

    Directive,
    Word,
    String,
    Char,
    Integer,
    Float,

    Skip,
    NewlineSkip,
}

#[derive(Clone, Copy)]
pub struct Token {
    pub kind: TokenKind,
    pub data: u32,
}

impl Token {
    fn len(&self, table: &StringTable) -> usize {
        match self.kind {
            TokenKind::Skip => return self.data as usize,
            TokenKind::NewlineSkip => return self.data as usize,

            TokenKind::Word => return table.names[self.data].len(),
            TokenKind::Directive => return table.names[self.data].len() + 1,
            TokenKind::String => return table.names[self.data].len() + 2,
            TokenKind::Char => return table.names[self.data].len() + 2,
            TokenKind::Integer => return table.names[self.data].len(),
            TokenKind::Float => return table.names[self.data].len(),

            TokenKind::Equal2 => return 2,
            TokenKind::LtEq => return 2,
            TokenKind::GtEq => return 2,
            TokenKind::And => return 2,
            TokenKind::Or => return 2,

            _ => return 1,
        }
    }
}

pub struct Parser {}

fn lex(table: &mut StringTable, file: u32, s: &str) -> Result<Pod<Token>, Error> {
    let mut tokens = Pod::new();
    let bytes = s.as_bytes();

    let mut index = 0;
    'outer: while let Some(&b) = bytes.get(index) {
        let begin = index;
        index += 1;

        'simple: loop {
            macro_rules! trailing_eq {
                ($e1:expr, $e2:expr) => {{
                    if let Some(b'=') = bytes.get(index) {
                        index += 1;

                        $e2
                    } else {
                        $e1
                    }
                }};
            }

            let kind = match b {
                b'(' => TokenKind::LParen,
                b')' => TokenKind::RParen,
                b'[' => TokenKind::LBracket,
                b']' => TokenKind::RBracket,
                b'{' => TokenKind::LBrace,
                b'}' => TokenKind::RBrace,
                b'.' => TokenKind::Dot,
                b',' => TokenKind::Comma,
                b':' => TokenKind::Colon,
                b';' => TokenKind::Semicolon,
                b'~' => TokenKind::Tilde,
                b'&' => TokenKind::Amp,
                b'^' => TokenKind::Caret,

                b'!' => trailing_eq!(TokenKind::Bang, TokenKind::NotEqual),
                b'=' => trailing_eq!(TokenKind::Equal, TokenKind::Equal2),
                b'<' => trailing_eq!(TokenKind::Lt, TokenKind::LtEq),
                b'>' => trailing_eq!(TokenKind::Gt, TokenKind::GtEq),

                // b'/' is handled separately because comments have more complex
                // syntax checking
                b'%' => TokenKind::Mod,
                b'*' => TokenKind::Star,
                b'+' => TokenKind::Plus,
                b'-' => TokenKind::Dash,

                _ => break 'simple,
            };

            tokens.push(Token { kind, data: 0 });
            continue 'outer;
        }

        if b == b'"' {
            let end = parse_string(file, bytes, index, b'"')?;
            let s = unsafe { core::str::from_utf8_unchecked(&bytes[index..(end - 1)]) };
            let data = table.add(s);

            index = end;

            let kind = TokenKind::String;
            tokens.push(Token { kind, data });
            continue 'outer;
        }

        if b == b'\'' {
            let end = parse_string(file, bytes, index, b'\'')?;
            let s = unsafe { core::str::from_utf8_unchecked(&bytes[index..(end - 1)]) };
            let data = table.add(s);

            index = end;

            let kind = TokenKind::Char;
            tokens.push(Token { kind, data });
            continue 'outer;
        }

        if b == b'/' {
            if let Some(b'/') = bytes.get(index) {
                index += 1;

                while let Some(&b) = bytes.get(index) {
                    index += 1;

                    if b == b'\n' {
                        break;
                    }
                }

                let kind = TokenKind::Skip;
                let data: u32 = expect((index - begin).try_into());
                tokens.push(Token { kind, data });
                continue 'outer;
            }

            let kind = TokenKind::Div;
            tokens.push(Token { kind, data: 0 });
            continue 'outer;
        }

        let is_alpha = (b >= b'a' && b <= b'z') || (b >= b'A' && b <= b'Z');
        if is_alpha || b == b'_' {
            while let Some(&b) = bytes.get(index) {
                let is_alpha = (b >= b'a' && b <= b'z') || (b >= b'A' && b <= b'Z');
                let is_num = b >= b'0' && b <= b'9';

                if is_alpha || b == b'_' || is_num {
                    index += 1;
                    continue;
                }

                break;
            }

            let s = unsafe { core::str::from_utf8_unchecked(&bytes[begin..index]) };
            let data = table.add(s);

            let kind = TokenKind::Word;
            tokens.push(Token { kind, data });
            continue 'outer;
        }

        let is_num = b >= b'0' && b <= b'9';
        if is_num {
            while let Some(&b) = bytes.get(index) {
                let is_num = b >= b'0' && b <= b'9';
                if b == b'_' || is_num {
                    index += 1;
                    continue;
                }

                break;
            }

            let s = unsafe { core::str::from_utf8_unchecked(&bytes[begin..index]) };
            let data = table.add(s);

            let kind = TokenKind::Integer;
            tokens.push(Token { kind, data });
            continue 'outer;
        }

        if b == b' ' || b == b'\t' || b == b'\r' || b == b'\n' {
            let mut has_newline = false;

            while let Some(&b) = bytes.get(index) {
                let is_newline = b == b'\r' || b == b'\n';
                if is_newline {
                    has_newline = true;
                    index += 1;

                    continue;
                }

                if b == b' ' || b == b'\t' {
                    index += 1;

                    continue;
                }

                break;
            }

            let kind = match has_newline {
                true => TokenKind::NewlineSkip,
                false => TokenKind::Skip,
            };

            let data: u32 = expect((index - begin).try_into());
            tokens.push(Token { kind, data });
            continue 'outer;
        }

        let error = Error::new("unrecognized token", file, begin..index);
        return Err(error);
    }

    return Ok(tokens);
}

fn parse_string(file: u32, bytes: &[u8], mut index: usize, terminator: u8) -> Result<usize, Error> {
    let begin = index;

    let mut escaped = false;
    while let Some(&b) = bytes.get(index) {
        index += 1;

        if b == b'\\' {
            escaped = true;
            continue;
        }

        if b == b'"' && !escaped {
            return Ok(index);
        }

        escaped = false;
    }

    return Err(Error::new(
        "failed to parse char or string",
        file,
        begin..index,
    ));
}

struct StringTable {
    allocator: BucketList,
    pub names: Pod<&'static str>,
    pub translate: HashMap<&'static str, u32>,
}

impl StringTable {
    pub fn new() -> Self {
        let mut table = Self {
            allocator: BucketList::new(),
            names: Pod::new(),
            translate: HashMap::new(),
        };

        let mut success = true;

        success = success && table.add("let") == Keyword::Let as u32;
        success = success && table.add("proc") == Keyword::Proc as u32;
        success = success && table.add("type") == Keyword::Type as u32;
        success = success && table.add("defer") == Keyword::Defer as u32;
        success = success && table.add("context") == Keyword::Context as u32;

        success = success && table.add("if") == Keyword::If as u32;
        success = success && table.add("else") == Keyword::Else as u32;
        success = success && table.add("match") == Keyword::Match as u32;

        success = success && table.add("continue") == Keyword::Continue as u32;
        success = success && table.add("break") == Keyword::Break as u32;
        success = success && table.add("for") == Keyword::For as u32;

        success = success && table.add("spawn") == Keyword::Spawn as u32;
        success = success && table.add("wait") == Keyword::Wait as u32;

        if !success {
            panic!("Rippo");
        }

        table
    }

    pub fn add(&mut self, s: &str) -> u32 {
        if let Some(id) = self.translate.get(s) {
            return *id;
        }

        let s = self.allocator.add_str(s);
        let id = self.names.len() as u32;

        self.translate.insert(s, id);
        self.names.push(s);

        return id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::*;
    use core::fmt::Write;

    #[test]
    fn test_parser() {
        let mut table = StringTable::new();
        let mut files = FileDb::new();

        let text = r#"
        let hello = wait slow()
        let a = 12

        print(hello, a)
        "#;

        if let Err(e) = files.add("data.liu", text) {
            panic!("{}", e);
        }

        let data = match lex(&mut table, 0, text) {
            Ok(data) => data,
            Err(e) => {
                let mut out = String::new();

                expect(e.render(&files, &mut out));

                eprintln!("{}\n", out);
                panic!("{:?}", e);
            }
        };
    }
}
