pub enum TedCommand<'a> {
    RedrawWindow,

    InsertText { index: usize, text: &'a str },
    DeleteText { begin: usize, end: usize },

    ForView { command: ViewCommand<'a> },
}

pub enum ViewCommand<'a> {
    Insert { text: &'a str },
    Delete,
    FlowCursor { file_index: usize },
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum TedEvent {
    Tick(usize),
}
