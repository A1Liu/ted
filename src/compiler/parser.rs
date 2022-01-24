use crate::compiler::types::*;
use crate::util::*;
use std::collections::hash_map::HashMap;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum TokenKind {
    Skip = 0,

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

fn lex(table: &mut StringTable, s: &str) -> Result<Pod<Token>, Error> {
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
            let end = parse_string(bytes, index, b'"')?;
            let s = unsafe { core::str::from_utf8_unchecked(&bytes[index..(end - 1)]) };
            let data = table.add(s);

            index = end;

            let kind = TokenKind::String;
            tokens.push(Token { kind, data });
            continue 'outer;
        }

        if b == b'\'' {
            let end = parse_string(bytes, index, b'\'')?;
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

        if b == b' ' || b == b'\t' || b == b'\r' || b == b'\n' {
            while let Some(&b) = bytes.get(index) {
                if b == b' ' || b == b'\t' || b == b'\r' || b == b'\n' {
                    index += 1;
                    continue;
                }

                break;
            }

            let kind = TokenKind::Skip;
            let data: u32 = expect((index - begin).try_into());
            tokens.push(Token { kind, data });
            continue 'outer;
        }

        let error = Error::new("unrecognized token", begin..index);
        return Err(error);
    }

    return Ok(tokens);
}

fn parse_string(bytes: &[u8], mut index: usize, terminator: u8) -> Result<usize, Error> {
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

    return Err(Error::new("failed to parse char or string", begin..index));
}

struct StringTable {
    allocator: BucketList,
    pub names: Pod<&'static str>,
    pub translate: HashMap<&'static str, u32>,
}

impl StringTable {
    pub fn new() -> Self {
        return Self {
            allocator: BucketList::new(),
            names: Pod::new(),
            translate: HashMap::new(),
        };
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
