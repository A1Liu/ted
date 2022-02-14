pub use mint::*;
pub use winit::event_loop::EventLoopProxy;
pub use winit::window::Window;

#[macro_use]
#[cfg(target_arch = "wasm32")]
pub mod wasm;

mod alloc_api;
mod bump;

#[macro_use]
mod pod;

mod hashref;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

pub use alloc_api::*;
pub use bump::*;
pub use hashref::*;
pub use pod::*;

pub trait AllocExt: Allocator {
    fn add_slice<T>(&self, slice: &[T]) -> &'static mut [T]
    where
        T: Copy,
    {
        use alloc::alloc::Layout;

        let len = slice.len();
        let size = core::mem::size_of::<T>() * len;
        let align = core::mem::align_of::<T>();
        unsafe {
            let layout = Layout::from_size_align_unchecked(size, align);
            let mut data = expect(self.allocate(layout));
            let block = data.as_mut().as_mut_ptr() as *mut T;
            let mut location = block;
            for &item in slice {
                core::ptr::write(location, item);
                location = location.add(1);
            }
            return core::slice::from_raw_parts_mut(block, len);
        }
    }

    fn add_str(&self, string: &str) -> &'static mut str {
        let string = string.as_bytes();
        return unsafe { core::str::from_utf8_unchecked_mut(self.add_slice(string)) };
    }
}

impl<A> AllocExt for A where A: Allocator {}

#[cfg(debug_assertions)]
pub fn expect<V, E>(res: Result<V, E>) -> V
where
    E: core::fmt::Debug,
{
    return res.unwrap();
}

#[cfg(not(debug_assertions))]
pub fn expect<V, E>(res: Result<V, E>) -> V {
    let err = match res {
        Ok(v) => return v,
        Err(err) => err,
    };

    panic!("Expected value");
}

pub fn unwrap<V>(opt: Option<V>) -> V {
    if let Some(v) = opt {
        return v;
    }

    panic!("Expected value");
}

pub type Rect = Vector2<u32>;

pub const fn new_rect(x: u32, y: u32) -> Rect {
    return Vector2 { x, y };
}

#[derive(Clone, Copy)]
pub struct CopyRange {
    pub start: usize,
    pub end: usize,
}

pub const fn r(start: usize, end: usize) -> CopyRange {
    return CopyRange { start, end };
}

impl CopyRange {
    #[inline(always)]
    pub fn len(&self) -> usize {
        return self.end - self.start;
    }
}

impl core::fmt::Debug for CopyRange {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        return write!(f, "{}..{}", self.start, self.end);
    }
}
