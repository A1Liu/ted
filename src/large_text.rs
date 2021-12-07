use crate::util::*;

const MAX_BUFFER_VIEW_SIZE: u16 = 4096;

#[derive(Clone, Copy)]
struct BufferView {
    begin: &'static [u8; 4096],
    content_size: u16,
}

// indices are split into 2 sections; the view index and the content index. Content
// index is the lower 12 bits.

// Could have large number of lines or really really long lines; So I guess use
// a BTree for both?

pub struct LargeFile {
    pub active: bool,
    file_cursor: usize,
    line_indices: BTree<usize>,
    lines: BTree<usize>,
    views: BTree<BufferView>,
}

const B: usize = 6;

// idk man im bad at naming things, sue me. This trait is for something that returns
// an integer which denotes its size.
trait Sizable {
    fn size(&self) -> usize;
}

impl Sizable for usize {
    fn size(&self) -> usize {
        return *self;
    }
}

impl Sizable for BufferView {
    fn size(&self) -> usize {
        return self.content_size as usize;
    }
}

// We're using the trick from Basic algo with the combining lists thing! From
// the 2-3 tree PSet. Very cute.
//                      - Albert Liu, Dec 06, 2021 Mon 19:11 EST
#[derive(Clone, Copy)]
struct Node {
    sum: usize,
    count: usize,
    parent: Idx, // For the root, I dont think this matters. Maybe it'll point to itself.
    kids: [Option<Idx>; B],
}

// Use SOA stuff eventually: https://github.com/lumol-org/soa-derive
// They even have a trait! Very cute.
#[derive(Clone)]
struct BTree<T>
where
    T: Sizable,
{
    elements: Vec<T>,
    nodes: Vec<Node>,
    root: usize,
    levels: usize,
}

impl<T> BTree<T>
where
    T: Sizable,
{
    fn new() -> Self {
        let root_node = Node {
            sum: 0,
            count: 0,
            parent: Idx::new(0),
            kids: [None; B],
        };

        return Self {
            elements: Vec::new(),
            nodes: vec![root_node],
            root: 0,
            levels: 0,
        };
    }

    fn get_idx(&self, index: usize) -> Option<Idx> {
        let mut node = self.nodes[self.root];
        if node.count >= index {
            return None;
        }

        let mut running = index;
        'outer: for _ in 0..self.levels {
            for child_idx in node.kids.into_iter().filter_map(|c| c) {
                let child = self.nodes[child_idx.get()];
                if running < child.count {
                    node = child;
                    continue 'outer;
                }

                running -= child.count;
            }
        }

        let index = node.kids.into_iter().filter_map(|c| c).nth(running);
        return Some(index.unwrap());
    }

    fn get_content_idx(&self, index: usize) -> Option<(usize, Idx)> {
        let mut node = self.nodes[self.root];
        if node.sum >= index {
            return None;
        }

        let mut running = index;
        'outer: for _ in 0..self.levels {
            for child_idx in node.kids.into_iter().filter_map(|c| c) {
                let child = self.nodes[child_idx.get()];
                if running < child.sum {
                    node = child;
                    continue 'outer;
                }

                running -= child.sum;
            }
        }

        for idx in node.kids.into_iter().filter_map(|c| c) {
            let size = self.elements[idx.get()].size();
            if running < size {
                return Some((running, idx));
            }

            running -= size;
        }

        unreachable!();
    }
}
