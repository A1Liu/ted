use crate::util::alloc_api::*;
use alloc::alloc::{alloc, dealloc, Layout};
use core::cell::Cell;
use std::ptr::NonNull;
use std::{cmp, mem, ptr, slice, str};

#[cfg(test)]
const BUCKET_SIZE: usize = 128;

#[cfg(not(test))]
const BUCKET_SIZE: usize = 2 * 1024 * 1024;

#[repr(C)]
pub struct BucketListInner {
    pub next: Cell<*const BucketListInner>,
    pub end: Cell<*const u8>,
    pub array_begin: (),
}

struct Bump {
    ptr: NonNull<u8>,
    next_bump: NonNull<u8>,
}

pub struct BucketList {
    inner: Cell<*const BucketListInner>,
}

impl BucketListInner {
    unsafe fn bump_size_align(bump: *const u8, end: *const u8, layout: Layout) -> Option<Bump> {
        let required_offset = bump.align_offset(layout.align());
        if required_offset == usize::MAX {
            return None;
        }

        let bump = bump.add(required_offset);
        let end_alloc = bump.add(layout.size());
        if end_alloc as usize > end as usize {
            return None;
        }

        let bump = Bump {
            ptr: NonNull::new_unchecked(bump as *mut u8),
            next_bump: NonNull::new_unchecked(end_alloc as *mut u8),
        };

        return Some(bump);
    }

    pub unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut list = self;

        loop {
            let array_begin = (&list.array_begin) as *const () as *const u8;
            let bucket_end = array_begin.add(BUCKET_SIZE);
            let mut bump = list.end.get();

            if let Some(Bump { ptr, next_bump }) = Self::bump_size_align(bump, bucket_end, layout) {
                list.end.set(next_bump.as_ptr());
                return ptr.as_ptr();
            }

            let next = list.next.get();
            if next.is_null() {
                break;
            }

            list = &*next;
        }

        let bucket_align = cmp::max(layout.align(), mem::align_of::<BucketListInner>());
        let inner_size = cmp::max(bucket_align, mem::size_of::<BucketListInner>());
        let bucket_size = inner_size + cmp::max(BUCKET_SIZE, layout.size());

        let next_layout = match Layout::from_size_align(bucket_size, bucket_align) {
            Ok(x) => x,
            Err(_) => return ptr::null_mut(),
        };

        let new_buffer = &mut *(alloc(next_layout) as *mut BucketListInner);
        let next_array_begin = &mut new_buffer.array_begin as *mut () as *mut u8;
        new_buffer.next = Cell::new(ptr::null_mut());
        new_buffer.end = Cell::new(next_array_begin.add(layout.size()));

        list.next.set(new_buffer);

        return next_array_begin;
    }
}

impl BucketList {
    pub fn new() -> Self {
        let bucket_align = mem::align_of::<BucketListInner>();
        let bucket_size = mem::size_of::<BucketListInner>() + BUCKET_SIZE;

        let inner = unsafe {
            let next_layout = Layout::from_size_align_unchecked(bucket_size, bucket_align);

            &mut *(alloc(next_layout) as *mut BucketListInner)
        };

        inner.next = Cell::new(ptr::null_mut());
        inner.end = Cell::new(&inner.array_begin as *const () as *const u8);

        let inner = Cell::new(inner as *const BucketListInner);

        return Self { inner };
    }

    pub fn clear(&mut self) {
        let mut bucket = self.inner.get();

        while !bucket.is_null() {
            let current = unsafe { &*bucket };
            let end = (&current.array_begin) as *const () as *const u8;
            current.end.set(end);
            bucket = current.next.get();
        }
    }
}

impl Drop for BucketList {
    fn drop(&mut self) {
        let mut bucket = self.inner.get();

        while !bucket.is_null() {
            let current = unsafe { &*bucket };

            let end = current.end.get() as usize;
            let begin = (&current.array_begin) as *const () as usize;
            let allocated_size = end - begin;

            let allocated_size = cmp::max(allocated_size, BUCKET_SIZE);
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
        let inner = unsafe { &*self.inner.get() };

        let ptr = unsafe { inner.alloc(layout) };

        let slice = unsafe { core::slice::from_raw_parts_mut(ptr, layout.size()) };
        let ptr = NonNull::new(slice).ok_or(AllocError)?;

        return Ok(ptr);
    }

    // deallocation doesn't do anything
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {}
}
