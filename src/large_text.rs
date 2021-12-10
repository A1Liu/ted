use crate::btree::*;

pub struct LargeFile {
    pub active: bool,
    file_cursor: usize,
    text: BTree<BufferView>,
}

struct BufferView {
    begin: Box<[u8; 4096]>,
    content_size: u16,
    newline_count: u16,
}

#[derive(Default, Clone, Copy)]
struct BufferInfo {
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

impl core::ops::Add<BufferInfo> for BufferInfo {
    type Output = Self;

    fn add(self, other: BufferInfo) -> BufferInfo {
        return BufferInfo {
            content_size: self.content_size + other.content_size,
            newline_count: self.newline_count + other.newline_count,
        };
    }
}

impl BTreeInfo for BufferInfo {}

impl BTreeItem for BufferView {
    type BTreeInfo = BufferInfo;
    fn get_info(&self) -> BufferInfo {
        return BufferInfo {
            content_size: self.content_size as usize,
            newline_count: self.newline_count as usize,
        };
    }
}
