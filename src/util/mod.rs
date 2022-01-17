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

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

pub use alloc_api::*;
pub use bump::*;
pub use pod::*;

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
