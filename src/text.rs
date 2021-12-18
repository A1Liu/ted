use btree::*;

pub struct File {
    data: BTree<TextBuffer>,
}

impl File {
    pub fn new() -> Self {
        let mut data = BTree::new();
        data.add(TextBuffer::new());

        return Self { data };
    }

    pub fn push(&mut self, c: char) {
        let mut data = [0u8; 4];
        let s = c.encode_utf8(&mut data);

        self.push_str(s);
    }

    pub fn push_str(&mut self, text: &str) {
        let last = self.data.last_idx().unwrap();
        let offset = self.data.get(last).unwrap().get_info().content_size;
        dbg!(offset);

        self.insert_at(last, offset, text);
    }

    pub fn insert(&mut self, idx: usize, text: &str) {
        let (idx, offset) = self.data.key_leq_idx(idx, BufferInfo::content).unwrap();

        self.insert_at(idx, offset, text);
    }

    pub fn last_line_begin(&self) -> usize {
        let lines = self.newlines();
        if lines == 0 {
            return 0;
        }

        return self.cursor_for_line(lines - 1).unwrap();
    }

    fn insert_at(&mut self, idx: ElemIdx, mut offset: usize, text: &str) {
        let remaining_chars = self.data.get_mut(idx, |buf| {
            let mut iter = text.chars();
            if buf.is_full() {
                return iter;
            }

            for c in &mut iter {
                let is_full = buf.insert(offset, c);
                offset += 1;

                if is_full {
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

        if buf_view.char_count > 0 {
            self.data.insert_after(idx, buf_view);
        }
    }

    pub fn newlines(&self) -> usize {
        return self.data.info().newline_count;
    }

    pub fn len(&self) -> usize {
        return self.data.info().content_size;
    }

    pub fn line_for_cursor(&self, idx: usize) -> Option<usize> {
        let (idx, remainder) = self.data.key_leq_idx(idx, BufferInfo::content)?;
        let lines_before = self.data.sum_until(idx)?.newline_count;
        let bytes = self.data[idx].buffer.as_bytes().iter();
        let lines = lines_before + bytes.take(remainder).filter(|&&b| b != b'\n').count();

        return Some(lines);
    }

    pub fn cursor_for_line(&self, line: usize) -> Option<usize> {
        let (idx, remainder) = self.data.key_leq_idx(line, BufferInfo::newlines)?;
        let cursor = self.data.sum_until(idx)?.content_size;

        return Some(cursor + remainder);
    }

    pub fn end_cursor_for_line(&self, line: usize) -> usize {
        return match self.cursor_for_line(line + 1) {
            Some(cursor) => cursor,
            None => self.len(),
        };
    }

    pub fn text_for_line<'a>(&'a self, line: usize) -> Option<LineIter<'a>> {
        let (idx, remainder) = self.data.key_leq_idx(line, BufferInfo::newlines)?;
        let idx = self.data.count_until(idx)?;

        return Some(LineIter {
            file: self,
            idx,
            buffer_idx: remainder,
        });
    }

    pub fn text_after_cursor<'a>(&'a self, cursor: usize) -> Option<TextIter<'a>> {
        let (idx, remainder) = self.data.key_idx(cursor, BufferInfo::content)?;
        let idx = self.data.count_until(idx)?;

        return Some(TextIter {
            file: self,
            idx,
            buffer_idx: remainder,
        });
    }
}

impl<'a> IntoIterator for &'a File {
    type Item = &'a str;
    type IntoIter = FileIter<'a>;

    fn into_iter(self) -> FileIter<'a> {
        return FileIter { file: self, idx: 0 };
    }
}

pub struct TextIter<'a> {
    file: &'a File,
    idx: usize,
    buffer_idx: usize,
}

impl<'a> Iterator for TextIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.file.data.get(self.idx)?;

        self.idx += 1;
        let bytes = &result.buffer.as_bytes()[self.buffer_idx..];
        self.buffer_idx = 0;

        return Some(unsafe { core::str::from_utf8_unchecked(bytes) });
    }
}

pub struct LineIter<'a> {
    file: &'a File,
    idx: usize,
    buffer_idx: usize,
}

impl<'a> Iterator for LineIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.file.data.get(self.idx)?;

        if result.newline_count == 0 {
            self.idx += 1;
            let bytes = &result.buffer.as_bytes()[self.buffer_idx..];
            self.buffer_idx = 0;

            return Some(unsafe { core::str::from_utf8_unchecked(bytes) });
        }

        self.idx = usize::MAX;
        let newline = result.buffer.as_bytes().iter().position(|&c| c == b'\n')?;
        let bytes = &result.buffer.as_bytes()[self.buffer_idx..newline];
        self.buffer_idx = 0;

        return Some(unsafe { core::str::from_utf8_unchecked(bytes) });
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
    char_count: u16,
    newline_count: u16,
}

impl TextBuffer {
    #[cfg(debug_assertions)]
    const MAX_LEN: usize = 8;

    #[cfg(not(debug_assertions))]
    const MAX_LEN: usize = 1024;

    pub fn new() -> Self {
        return Self {
            buffer: String::new(),
            char_count: 0,
            newline_count: 0,
        };
    }

    pub fn is_full(&self) -> bool {
        return self.buffer.len() >= TextBuffer::MAX_LEN;
    }

    pub fn insert(&mut self, idx: usize, c: char) -> bool {
        if self.buffer.len() == 0 {
            self.buffer.reserve(TextBuffer::MAX_LEN + 4);
        }

        self.buffer.insert(idx, c);
        self.char_count += 1;
        if c == '\n' {
            self.newline_count += 1;
        }

        return self.is_full();
    }

    pub fn push(&mut self, c: char) -> bool {
        if self.buffer.len() == 0 {
            self.buffer.reserve(TextBuffer::MAX_LEN + 4);
        }

        self.buffer.push(c);
        self.char_count += 1;
        if c == '\n' {
            self.newline_count += 1;
        }

        return self.is_full();
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
            content_size: self.char_count as usize,
            newline_count: self.newline_count as usize,
        };
    }
}
