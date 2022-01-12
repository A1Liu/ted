use core::num::NonZeroUsize;
pub use mint::*;
pub use winit::event_loop::EventLoopProxy;
pub use winit::window::Window;

#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen::JsCast;

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

pub fn new_rect(x: u32, y: u32) -> Rect {
    return Vector2 { x, y };
}

#[cfg(target_arch = "wasm32")]
pub type JsFunc = dyn 'static + FnMut() -> Result<(), JsValue>;

#[cfg(target_arch = "wasm32")]
pub fn enclose(f: impl 'static + FnMut() -> Result<(), JsValue>) -> Closure<JsFunc> {
    return Closure::wrap(Box::new(f) as Box<JsFunc>);
}

#[cfg(target_arch = "wasm32")]
pub fn get_canvas() -> Result<web_sys::HtmlCanvasElement, JsValue> {
    let window = unwrap(web_sys::window());
    let document = unwrap(window.document());
    let canvas = unwrap(document.get_element_by_id("canvas"));
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    return Ok(canvas);
}
