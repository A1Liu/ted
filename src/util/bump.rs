use super::pod::*;
use super::unwrap;
use crate::util::alloc_api::*;
use alloc::alloc::{alloc, dealloc, Layout};
use core::cell::Cell;
use std::ptr::NonNull;
use std::{cmp, mem, ptr, slice, str};

#[cfg(test)]
const DEFAULT_BUCKET_SIZE: usize = 128;

#[cfg(not(test))]
const DEFAULT_BUCKET_SIZE: usize = 2 * 1024 * 1024;

#[derive(Clone, Copy)]
struct Bump {
    ptr: NonNull<u8>,
    current: NonNull<u8>,
    layout: Layout,
}

const DANGLING: NonNull<u8> = NonNull::dangling();

const EMPTY_BUMP: Bump = Bump {
    ptr: DANGLING,
    current: DANGLING,
    layout: unsafe { Layout::from_size_align_unchecked(0, 1) },
};

impl Bump {
    fn new(layout: Layout) -> Bump {
        let ptr = unsafe { alloc(layout) };
        let ptr = unwrap(NonNull::new(ptr));

        return Bump {
            ptr,
            current: ptr,
            layout,
        };
    }

    fn alloc(&mut self, layout: Layout) -> Option<*mut u8> {
        if self.layout.size() == 0 {
            return None;
        }

        if layout.align() > 8 {
            panic!("Not handled");
        }

        let required_offset = self.current.as_ptr().align_offset(layout.align());
        if required_offset == usize::MAX {
            return None;
        }

        unsafe {
            let alloc_begin = self.current.as_ptr().add(required_offset);
            let alloc_end = alloc_begin.add(layout.size());
            let bump_end = self.ptr.as_ptr() as usize + self.layout.size();

            if alloc_end as usize <= bump_end {
                self.current = NonNull::new_unchecked(alloc_end);

                return Some(alloc_begin as *mut u8);
            }

            return None;
        }
    }
}

pub struct BucketList {
    allocations: Cell<Pod<Bump>>,
    bump: Cell<Bump>,
}

impl BucketList {
    #[inline(always)]
    pub fn new() -> Self {
        return Self {
            allocations: Cell::new(Pod::new()),
            bump: Cell::new(EMPTY_BUMP),
        };
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let layout = match Layout::from_size_align(capacity, 8) {
            Ok(layout) => layout,
            Err(e) => panic!("failed to make Layout"),
        };

        let mut allocations = Pod::new();

        let bump = Bump::new(layout);

        return Self {
            allocations: Cell::new(allocations),
            bump: Cell::new(bump),
        };
    }
}

unsafe impl Send for BucketList {}

impl Drop for BucketList {
    fn drop(&mut self) {
        let bump = self.bump.get();
        if bump.layout.size() == 0 {
            return;
        }

        let allocations = self.allocations.replace(Pod::new());

        for bump in allocations {
            unsafe {
                dealloc(bump.ptr.as_ptr(), bump.layout);
            }
        }

        unsafe {
            dealloc(bump.ptr.as_ptr(), bump.layout);
        }
    }
}

unsafe impl Allocator for BucketList {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let mut bump = self.bump.get();

        let ptr = bump.alloc(layout).unwrap_or_else(|| unsafe {
            let mut allocations = self.allocations.replace(Pod::new());

            let size = std::cmp::max(layout.size(), DEFAULT_BUCKET_SIZE);
            let layout = Layout::from_size_align_unchecked(size, layout.align());

            if bump.layout.size() > 0 {
                allocations.push(bump);
            }

            bump = Bump::new(layout);

            self.allocations.replace(allocations);

            return unwrap(bump.alloc(layout));
        });

        let slice = unsafe { core::slice::from_raw_parts_mut(ptr, layout.size()) };
        let ptr = NonNull::new(slice).ok_or(AllocError)?;

        self.bump.set(bump);

        return Ok(ptr);
    }

    // deallocation doesn't do anything
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {}
}
