use crate::large_text::LargeFile;

pub struct Files {
    files: Vec<LargeFile>,
    text_views: Vec<TextView>,
}

pub struct TextView {
    pub active: bool,
    line_indices: Vec<usize>,
    data: String,
}
