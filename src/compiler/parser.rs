use crate::util::*;
use std::collections::hash_map::HashMap;

#[repr(u8)]
enum TokenKind {
    Equal,
    Equal2,
}

struct Token {
    kind: TokenKind,
    string: u32,
}

// fn lex(table: &mut StringTable, s: &str) -> Pod<Token> {}

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
