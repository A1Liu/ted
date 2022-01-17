use crate::util::*;
use std::collections::hash_map::HashMap;

pub enum GonValue<'a> {
    Object(HashMap<&'a str, GonValue<'a>>),
    Str(&'a str),
    String(String),
}

enum Token<'a> {
    Str(&'a str),
    String(String),

    Equal,
    Comma,
    Colon,

    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
}

fn symbol_token<'a>(b: u8) -> Option<(Token<'a>, bool)> {
    let tok = match b {
        b'=' => (Token::Equal, true),
        b',' => (Token::Comma, true),
        b':' => (Token::Colon, true),
        b'{' => (Token::OpenBrace, false),
        b'}' => (Token::CloseBrace, false),
        b'[' => (Token::OpenBracket, false),
        b']' => (Token::CloseBracket, false),
        _ => return None,
    };

    return Some(tok);
}

fn slice_token<'a>(string: &'a [u8]) -> Token<'a> {
    return Token::Str(unsafe { core::str::from_utf8_unchecked(string) });
}

fn string_token<'a>(string: &[u8]) -> Token<'a> {
    return Token::String(unsafe { core::str::from_utf8_unchecked(string).to_string() });
}

// fn string_token<'a>(string: Cow<'a, [u8]>) -> Token<'a> {
//     match string {
//         Cow::Borrowed(s) => return,
//         Cow::Owned(s) => return Token::String(unsafe { String::from_utf8_unchecked(s) }),
//     }
// }

fn tokenize<'a>(data: &'a str) -> Vec<Token<'a>> {
    let mut tokens = Vec::new();

    let bytes = data.as_bytes();
    let mut index = 0;

    let mut current_token_begin = None;
    let mut current_string: Pod<u8> = Pod::new();

    'outer: while let Some(&b) = bytes.get(index) {
        if b == b'#' {
            if let Some(begin) = current_token_begin.take() {
                tokens.push(slice_token(&bytes[begin..index]));
            }

            while let Some(&b) = bytes.get(index) {
                index += 1;

                if b == b'\n' {
                    continue 'outer;
                }
            }
        }

        if b == b' ' || b == b'\n' || b == b'\r' || b == b'\t' {
            if let Some(begin) = current_token_begin.take() {
                tokens.push(slice_token(&bytes[begin..index]));
            }
        }

        if b == b'"' {
            if let Some(begin) = current_token_begin.take() {
                tokens.push(slice_token(&bytes[begin..index]));
            }

            index += 1;

            let begin = index;
            let mut is_escape = false;

            while let Some(&b) = bytes.get(index) {
                if b == b'\\' {
                    is_escape = true;
                    break;
                }

                if b == b'"' {
                    break;
                }

                index += 1;
            }

            let done_so_far = &bytes[begin..index];
            index += 1;

            if !is_escape {
                tokens.push(slice_token(done_so_far));
                continue;
            }

            current_string.reserve(done_so_far.len());
            for &byte in done_so_far {
                current_string.push(byte);
            }

            while let Some(&b) = bytes.get(index) {
                if is_escape {
                    // TODO support more escapes I guess? Right now this
                    // supports \n \' \" and a few other things.
                    //                  - Albert Liu, Jan 17, 2022 Mon 02:17 EST
                    match b {
                        b'n' => current_string.push(b'\n'),

                        _ => current_string.push(b),
                    }
                }

                match b {
                    b'"' => break,
                    b'\\' => is_escape = true,
                    _ => current_string.push(b),
                }

                index += 1;
            }

            tokens.push(string_token(&*current_string));
            index += 1;

            current_string.clear();
            continue;
        }

        if let Some((tok, ignored)) = symbol_token(b) {
            if let Some(begin) = current_token_begin.take() {
                tokens.push(slice_token(&bytes[begin..index]));
            }

            if !ignored {
                tokens.push(tok);
            }

            index += 1;
            continue;
        }

        current_token_begin.get_or_insert(index);
        index += 1;
    }

    if let Some(begin) = current_token_begin.take() {
        tokens.push(slice_token(&bytes[begin..index]));
    }

    return tokens;
}
