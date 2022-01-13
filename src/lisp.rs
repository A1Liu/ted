use crate::util::*;

// TODO IDK if lisp is the right decision for this. I saw a blog post that gave a tutorial
// on how to do a lisp interpreter, so that's what I'm going with. I couldn't care less
// about what langauge is used to program this part of the editor, especially right now.
//
// This was the blog post: https://justine.lol/sectorlisp2/
//
// I can't figure out what the hell the blog post's algorithm actually does,
// so I'm just gonna implement it myself.
//
//                                  - Albert Liu, Jan 12, 2022 Wed 23:44 EST

const NULL: i32 = -1;
const EQ: i32 = NULL - 1;
const CONS: i32 = EQ - 1;
const ATOM: i32 = CONS - 1;
const CAR: i32 = ATOM - 1;
const CDR: i32 = CAR - 1;
const DEFINE: i32 = CDR - 1;

pub struct Interp {
    atom_data: Vec<u8>,
    atoms: Vec<(u32, u32)>,

    // indexes into memory
    defines: Vec<u32>,
    memory: Vec<i32>,
}

impl Interp {
    pub fn new() -> Self {
        let mut s = Self {
            atom_data: Vec::new(),
            atoms: Vec::new(),
            defines: Vec::new(),
            memory: Vec::new(),
        };

        s.add_atom("null");
        s.add_atom("=");
        s.add_atom("cons");
        s.add_atom("atom");
        s.add_atom("car");
        s.add_atom("cdr");
        s.add_atom("define");

        return s;
    }

    fn add_atom(&mut self, atom: &str) -> i32 {
        let begin = self.atom_data.len() as u32;
        self.atom_data.extend_from_slice(atom.as_bytes());

        let end = self.atom_data.len() as u32;
        let atom = self.atoms.len() as i32;

        self.atoms.push((begin, end));

        return -atom - 1;
    }

    // tail
    fn cdr(&self, x: i32) -> i32 {
        if x < 0 {
            panic!("tried to call car on something that wasn't a list");
        }

        return *unwrap(self.memory.get(x as usize + 1));
    }

    // first element
    fn car(&self, x: i32) -> i32 {
        if x < 0 {
            panic!("tried to call car on something that wasn't a list");
        }

        return *unwrap(self.memory.get(x as usize));
    }
}
