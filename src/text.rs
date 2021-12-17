use crate::btree::*;

pub struct File {
    data: BTree<TextBuffer>,
}

impl File {
    pub fn new() -> Self {
        return Self { data: BTree::new() };
    }

    pub fn insert(&mut self, idx: usize, text: &str) {
        let res = self.data.key_leq_idx(idx, BufferInfo::content);
        let (idx, _) = match res {
            None => (self.data.add(TextBuffer::new()), 0),
            Some(res) => res,
        };

        let remaining_chars = self.data.get_mut(idx, |buf_view| {
            let mut iter = text.chars();
            for c in &mut iter {
                if buf_view.push(c) {
                    break;
                }
            }

            return iter;
        });

        let mut buf_view = TextBuffer::new();
        let mut idx = idx;
        for c in &mut remaining_chars.unwrap() {
            if buf_view.push(c) {
                idx = self.data.insert_after(idx, buf_view);
                buf_view = TextBuffer::new();
            }
        }
    }

    // pub fn line_number(&self, idx: usize) {}
}

impl<'a> IntoIterator for &'a File {
    type Item = &'a str;
    type IntoIter = FileIter<'a>;

    fn into_iter(self) -> FileIter<'a> {
        return FileIter { file: self, idx: 0 };
    }
}

pub struct FileIter<'a> {
    file: &'a File,
    idx: usize,
}

impl<'a> Iterator for FileIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.file.data.get(self.idx);
        self.idx += 1;
        return result.map(|buf_view| &*buf_view.buffer);
    }
}

struct TextBuffer {
    buffer: String,
    newline_count: u16,
}

impl TextBuffer {
    const MAX_LEN: usize = 1024;

    pub fn new() -> Self {
        return Self {
            buffer: String::new(),
            newline_count: 0,
        };
    }

    pub fn push(&mut self, c: char) -> bool {
        if self.buffer.len() == 0 {
            self.buffer.reserve(TextBuffer::MAX_LEN);
        }

        self.buffer.push(c);
        if c == '\n' {
            self.newline_count += 1;
        }

        return self.buffer.len() >= TextBuffer::MAX_LEN;
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

impl BTreeItem for TextBuffer {
    type Info = BufferInfo;
    fn get_info(&self) -> BufferInfo {
        return BufferInfo {
            content_size: self.buffer.len(),
            newline_count: self.newline_count as usize,
        };
    }
}
