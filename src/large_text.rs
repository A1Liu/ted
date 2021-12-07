use crate::util::Idx;

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
    children: [Option<Idx>; B],
}

// Use SOA stuff eventually: https://github.com/lumol-org/soa-derive
// They even have a trait! Very cute.
#[derive(Clone)]
struct BTree<T>
where
    T: Sizable,
{
    children: Vec<T>,
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
            children: [None; B],
        };

        return Self {
            children: Vec::new(),
            nodes: vec![root_node],
            root: 0,
            levels: 1,
        };
    }
}
