#![no_std]

extern crate alloc;

use alloc::vec::Vec;

pub trait StringSize {
    const SIZE: usize;
}

pub struct StringU8;

pub struct String1 {
    bytes: Vec<u8>,
}

impl String1 {
    pub fn new() -> Self {
        return Self { bytes: Vec::new() };
    }

    pub fn as_bytes(&self) -> &[u8] {
        return &self.bytes;
    }
}
