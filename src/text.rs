use crate::btree::*;

pub struct File {
    file_cursor: usize,
    data: BTree<BufferView>,
}

impl File {
    pub fn new() -> Self {
        return Self {
            file_cursor: 0,
            data: BTree::new(),
        };
    }

    pub fn insert(&mut self, text: &str) {
        let res = self.data.key_leq_idx(self.file_cursor, BufferInfo::content);
        let (buffer, rem) = res.unwrap_or_else(|| (self.data.add(BufferView::new()), 0));

        self.file_cursor += text.len();
    }
}

pub struct BufferView {
    buffer: String,
    newline_count: u16,
}

impl BufferView {
    pub fn new() -> Self {
        return Self {
            buffer: String::new(),
            newline_count: 0,
        };
    }
}

#[derive(Default, Clone, Copy)]
pub struct BufferInfo {
    content_size: usize,
    newline_count: usize,
}

impl BufferInfo {
    fn content(self: BufferInfo) -> usize {
        return self.content_size;
    }

    fn newlines(self: BufferInfo) -> usize {
        return self.newline_count;
    }
}

impl BTreeInfo for BufferInfo {
    fn add(self, other: Self) -> Self {
        return BufferInfo {
            content_size: self.content_size + other.content_size,
            newline_count: self.newline_count + other.newline_count,
        };
    }
}

impl BTreeItem for BufferView {
    type Info = BufferInfo;
    fn get_info(&self) -> BufferInfo {
        return BufferInfo {
            content_size: self.buffer.len(),
            newline_count: self.newline_count as usize,
        };
    }
}
