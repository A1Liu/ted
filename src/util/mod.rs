pub use aliu::*;
pub use mint::*;
pub use winit::event_loop::EventLoopProxy;
pub use winit::window::Window;

#[macro_use]
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

pub type Rect = Vector2<u32>;

pub const fn new_rect(x: u32, y: u32) -> Rect {
    return Vector2 { x, y };
}
