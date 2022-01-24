use crate::util::*;

// TODO Placeholder system. Eventually we'll flesh this out maybe. For now, 'tis
// a simple thing with a bit of needless complexity
//                              - Albert Liu, Jan 23, 2022 Sun 22:21 EST

pub enum Error {
    Message(ErrorMessage),
}

pub struct ErrorMessage {
    message: String,
    file: u32,
    range: CopyRange,
}

impl Error {
    pub fn new(s: impl Into<String>, range: CopyRange) -> Self {
        return Self::Message(ErrorMessage {
            message: s.into(),
            file: 0,
            range,
        });
    }

    pub fn append_messages(self, messages: &mut Vec<ErrorMessage>) {
        match self {
            Self::Message(message) => messages.push(message),
        }
    }
}
