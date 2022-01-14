pub use mint::*;
pub use winit::event_loop::EventLoopProxy;
pub use winit::window::Window;

#[macro_use]
#[cfg(target_arch = "wasm32")]
pub mod wasm;

mod alloc_api;
mod pod;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

pub use alloc_api::*;
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
