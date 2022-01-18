use crate::util::*;
use alloc::alloc::{alloc, dealloc, Layout, LayoutError};
use core::ptr::NonNull;

#[derive(Debug)]
pub struct AllocError;

// The rust version isn't out of nightly yet
pub unsafe trait Allocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError>;

    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let mut ptr = self.allocate(layout)?;
        unsafe {
            let s_ptr = ptr.as_mut();
            s_ptr.as_mut().as_mut_ptr().write_bytes(0, s_ptr.len());
        }
        Ok(ptr)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout);

    unsafe fn grow(
        &self,
        mut ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`"
        );

        let mut new_ptr = self.allocate(new_layout)?;

        let (s_ptr, s_new_ptr) = (ptr.as_mut(), new_ptr.as_mut());
        core::ptr::copy_nonoverlapping(s_ptr, s_new_ptr.as_mut_ptr(), old_layout.size());
        self.deallocate(ptr, old_layout);

        Ok(new_ptr)
    }

    unsafe fn grow_zeroed(
        &self,
        mut ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`"
        );

        let mut new_ptr = self.allocate_zeroed(new_layout)?;

        let (s_ptr, s_new_ptr) = (ptr.as_mut(), new_ptr.as_mut());
        core::ptr::copy_nonoverlapping(s_ptr, s_new_ptr.as_mut_ptr(), old_layout.size());
        self.deallocate(ptr, old_layout);

        Ok(new_ptr)
    }

    unsafe fn shrink(
        &self,
        mut ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() <= old_layout.size(),
            "`new_layout.size()` must be smaller than or equal to `old_layout.size()`"
        );

        let mut new_ptr = self.allocate(new_layout)?;

        let (s_ptr, s_new_ptr) = (ptr.as_mut(), new_ptr.as_mut());
        core::ptr::copy_nonoverlapping(s_ptr, s_new_ptr.as_mut_ptr(), new_layout.size());
        self.deallocate(ptr, old_layout);

        Ok(new_ptr)
    }

    fn by_ref(&self) -> &Self
    where
        Self: Sized,
    {
        self
    }
}

#[derive(Clone, Copy)]
pub struct Global;

unsafe impl Allocator for Global {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        unsafe {
            let mut data = alloc(layout);

            let data = unwrap(data.as_mut());
            let data = core::slice::from_raw_parts_mut(data, layout.size());

            return Ok(NonNull::new_unchecked(data));
        }
    }

    unsafe fn deallocate(&self, mut ptr: NonNull<u8>, layout: Layout) {
        dealloc(ptr.as_mut(), layout);
    }
}

unsafe impl<A> Allocator for &A
where
    A: Allocator + ?Sized,
{
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        (**self).allocate(layout)
    }

    #[inline]
    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        (**self).allocate_zeroed(layout)
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        (**self).deallocate(ptr, layout)
    }

    #[inline]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        (**self).grow(ptr, old_layout, new_layout)
    }

    #[inline]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        (**self).grow_zeroed(ptr, old_layout, new_layout)
    }

    #[inline]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        (**self).shrink(ptr, old_layout, new_layout)
    }
}
