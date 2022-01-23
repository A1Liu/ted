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

fn lex(table: &mut StringTable, s: &str) -> Pod<Token> {
    let mut tokens = Pod::new();
    let bytes = s.as_bytes();

    let mut index = 0;
    'outer: while let Some(&b) = bytes.get(index) {
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

                b'%' => TokenKind::Mod,
                b'*' => TokenKind::Star,
                b'+' => TokenKind::Plus,
                b'-' => TokenKind::Dash,

                _ => break 'simple,
            };

            tokens.push(Token { kind, data: 0 });

            continue 'outer;
        }

        if b == b'"' {}

        if b == b'/' {}

        if (b >= b'a' && b <= b'z') || b == b'_' {}
    }

    return tokens;
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
