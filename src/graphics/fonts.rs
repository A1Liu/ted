use crate::util::*;
use mint::Point2;
use std::collections::hash_map::HashMap;

const MAX_ATLAS_WIDTH: u32 = 4096;
const COURIER: &[u8] = core::include_bytes!("./cour.ttf");

// These affect how the font looks I think? I'm not really sure tbh.
//                                  - Albert Liu, Dec 11, 2021 Sat 22:44 EST
const SIZE: usize = 128; // some kind of font thing idk.

pub const EMPTY_GLYPH: Glyph = Glyph {
    top_left_1: Point2 { x: 0, y: 0 },
    top_right_1: Point2 { x: 0, y: 0 },
    bot_left_1: Point2 { x: 0, y: 0 },
    top_right_2: Point2 { x: 0, y: 0 },
    bot_left_2: Point2 { x: 0, y: 0 },
    bot_right_2: Point2 { x: 0, y: 0 },
};

const DEFAULT_CHARS: &'static str = core::concat!(
    " ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "abcdefghijklmnopqrstuvwxyz",
    "0123456789",
    r#"`~!@#$%^&*()_+-=[]{};':",.<>/?\|"#
);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Glyph {
    // each glyph is 2 trianges of 3 points each
    top_left_1: Point2<u32>,
    top_right_1: Point2<u32>,
    bot_left_1: Point2<u32>,
    top_right_2: Point2<u32>,
    bot_left_2: Point2<u32>,
    bot_right_2: Point2<u32>,
}

impl PartialEq for Glyph {
    fn eq(&self, other: &Self) -> bool {
        return self.top_left_1 == other.top_left_1;
    }
}
