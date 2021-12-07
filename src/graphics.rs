pub enum SourceId {
    File { id: usize },
    Text { id: usize },
}

pub struct Canvas {
    pub source: SourceId,
    pub screen_id: usize,
    pub width: usize,
    pub height: usize,
}

pub struct Graphics {
    screens: Vec<Canvas>,
}
