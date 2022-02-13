use crate::util::alloc_api::*;
use alloc::alloc::{alloc, dealloc, Layout};
use core::cell::Cell;
use std::ptr::NonNull;
use std::{cmp, mem, ptr, slice, str};

#[cfg(test)]
const DEFAULT_BUCKET_SIZE: usize = 128;

#[cfg(not(test))]
const DEFAULT_BUCKET_SIZE: usize = 2 * 1024 * 1024;

#[repr(C)]
pub struct BucketListInner {
    pub next: Cell<*const BucketListInner>,
    pub current: Cell<*const u8>,
    pub end: *const u8,
    pub array_begin: (),
}

struct Bump {
    ptr: NonNull<u8>,
    next_bump: NonNull<u8>,
}

pub struct BucketList {
    begin: Cell<*const BucketListInner>,
    current: Cell<*const BucketListInner>,
}

impl BucketListInner {
    pub unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() > 8 {
            panic!("Not handled");
        }

        let mut list = self;

        loop {
            let current = list.current.get();
            let required_offset = current.align_offset(layout.align());
            if required_offset == usize::MAX {
                let next = list.next.get();
                if next.is_null() {
                    break;
                }

                list = &*next;
                continue;
            }

            let bump = current.add(required_offset);
            let end_alloc = bump.add(layout.size());
            if end_alloc as usize <= list.end as usize {
                list.current.set(end_alloc);

                return bump as *mut u8;
            }

            let next = list.next.get();
            if next.is_null() {
                break;
            }

            list = &*next;
        }

        let bucket_align = cmp::max(layout.align(), mem::align_of::<BucketListInner>());
        let inner_size = cmp::max(bucket_align, mem::size_of::<BucketListInner>());

        let current_end = list.end as usize;
        let current_begin = (&list.array_begin) as *const () as *const u8;
        let current_size = current_end - current_begin as usize;

        let bucket_size = inner_size + cmp::max(current_size * 3 / 2, layout.size());

        let next_layout = match Layout::from_size_align(bucket_size, bucket_align) {
            Err(_) => return ptr::null_mut(),
            Ok(x) => x,
        };

        let new_buffer_ptr = alloc(next_layout);
        let new_buffer = &mut *(new_buffer_ptr as *mut BucketListInner);
        let next_array_begin = (&mut new_buffer.array_begin) as *mut () as *mut u8;

        new_buffer.next = Cell::new(ptr::null_mut());
        new_buffer.current = Cell::new(next_array_begin.add(layout.size()));
        new_buffer.end = new_buffer_ptr.add(next_layout.size());

        list.next.set(new_buffer);

        return next_array_begin;
    }
}

impl BucketList {
    #[inline(always)]
    pub fn new() -> Self {
        return Self::with_capacity(DEFAULT_BUCKET_SIZE);
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let bucket_align = mem::align_of::<BucketListInner>();
        let capacity = ((capacity - 1) / bucket_align + 1) * bucket_align;
        let bucket_size = mem::size_of::<BucketListInner>() + capacity;

        let inner = unsafe {
            let next_layout = Layout::from_size_align_unchecked(bucket_size, bucket_align);

            &mut *(alloc(next_layout) as *mut BucketListInner)
        };

        let inner_ptr = (inner) as *mut BucketListInner as *mut u8 as *const u8;

        inner.next = Cell::new(ptr::null_mut());
        inner.current = Cell::new((&inner.array_begin) as *const () as *const u8);
        inner.end = unsafe { inner_ptr.add(bucket_size) };

        let inner = inner as *const BucketListInner;
        let begin = Cell::new(inner);
        let current = Cell::new(inner);

        return Self { begin, current };
    }

    pub fn clear(&mut self) {
        let begin = self.begin.get();
        let mut bucket = begin;

        while !bucket.is_null() {
            let current = unsafe { &*bucket };
            let begin = (&current.array_begin) as *const () as *const u8;
            current.current.set(begin);
            bucket = current.next.get();
        }

        self.current.set(begin);
    }
}

unsafe impl Send for BucketList {}
unsafe impl Sync for BucketList {}

impl Drop for BucketList {
    fn drop(&mut self) {
        let mut bucket = self.begin.get();

        while !bucket.is_null() {
            let current = unsafe { &*bucket };

            let end = current.end as usize;
            let begin = bucket as usize;
            let allocated_size = end - begin;

            let allocated_size = allocated_size + mem::size_of::<BucketListInner>();
            let next_bucket = current.next.get();
            unsafe {
                dealloc(
                    bucket as *mut u8,
                    Layout::from_size_align_unchecked(allocated_size, 1),
                );
            }
            bucket = next_bucket;
        }
    }
}

unsafe impl Allocator for BucketList {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let inner = unsafe { &*self.current.get() };

        let ptr = unsafe { inner.alloc(layout) };

        let next = inner.next.get();
        if !next.is_null() {
            self.current.set(inner.next.get());
        }

        let slice = unsafe { core::slice::from_raw_parts_mut(ptr, layout.size()) };
        let ptr = NonNull::new(slice).ok_or(AllocError)?;

        return Ok(ptr);
    }

    // deallocation doesn't do anything
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {}
}
