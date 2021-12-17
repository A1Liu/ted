#![no_std]

extern crate alloc;

use alloc::vec::Vec;

pub trait StringSize {
    const SIZE: usize;
}

pub struct StringU8;

impl StringSize for StringU8 {
    const SIZE: usize = 1;
}

pub struct IString<T>
where
    T: StringSize,
{
    bytes: Vec<u8>,
    phantom: core::marker::PhantomData<T>,
}

impl<T> IString<T>
where
    T: StringSize,
{
    pub fn new() -> Self {
        return Self {
            bytes: Vec::new(),
            phantom: core::marker::PhantomData,
        };
    }

    pub fn as_bytes(&self) -> &[u8] {
        return &self.bytes;
    }
}
