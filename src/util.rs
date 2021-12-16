use core::num::NonZeroUsize;
pub use mint::*;
use winit::event_loop::EventLoopProxy;

#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen::JsCast;

pub fn expect<V, E>(res: Result<V, E>) -> V
where
    E: core::fmt::Debug,
{
    #[cfg(debug_assertions)]
    {
        return res.unwrap();
    }

    #[cfg(not(debug_assertions))]
    {
        let err = match res {
            Ok(v) => return v,
            Err(err) => err,
        };

        panic!("Expected value");
    }
}

pub fn unwrap<V>(opt: Option<V>) -> V {
    #[cfg(debug_assertions)]
    {
        return opt.unwrap();
    }

    #[cfg(not(debug_assertions))]
    {
        if let Some(v) = opt {
            return v;
        }

        panic!("Expected value");
    }
}

#[derive(Clone, Copy, Default)]
pub struct Rect {
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(width: u32, height: u32) -> Self {
        return Self { width, height };
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Idx(NonZeroUsize);

impl Into<usize> for Idx {
    fn into(self) -> usize {
        return self.get();
    }
}

impl std::fmt::Debug for Idx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.0);
    }
}

impl Idx {
    #[inline(always)]
    pub fn new(i: usize) -> Idx {
        // this will panic anyways later on in the pipeline
        return Idx(unsafe { NonZeroUsize::new_unchecked(i + 1) });
    }

    #[inline(always)]
    pub fn get(self) -> usize {
        return self.0.get() - 1;
    }
}

#[cfg(target_arch = "wasm32")]
pub type JsFunc = dyn 'static + FnMut() -> Result<(), JsValue>;

#[cfg(target_arch = "wasm32")]
pub fn enclose(f: impl 'static + FnMut() -> Result<(), JsValue>) -> Closure<JsFunc> {
    return Closure::wrap(Box::new(f) as Box<JsFunc>);
}
