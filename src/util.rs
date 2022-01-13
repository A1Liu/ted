use core::num::NonZeroUsize;
pub use mint::*;
pub use winit::event_loop::EventLoopProxy;
pub use winit::window::Window;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[macro_use]
#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use super::*;

    pub use wasm_bindgen::prelude::*;
    pub use wasm_bindgen::JsCast;

    pub type JsFunc = dyn 'static + FnMut() -> Result<(), JsValue>;

    pub fn enclose(f: impl 'static + FnMut() -> Result<(), JsValue>) -> Closure<JsFunc> {
        return Closure::wrap(Box::new(f) as Box<JsFunc>);
    }

    pub fn get_canvas() -> Result<web_sys::HtmlCanvasElement, JsValue> {
        let window = unwrap(web_sys::window());
        let document = unwrap(window.document());
        let canvas = unwrap(document.get_element_by_id("canvas"));
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
        return Ok(canvas);
    }

    #[macro_export]
    macro_rules! out {
        ($str:expr, $( $e:expr ),+ ) => {{
            #[cfg(debug_assertions)]
            {
                out!(@CLEAN, core::concat!("[{}:{}]: ", $str, "\n"), file!(), line!(), $( $e ),+ );
            }
        }};
        (@CLEAN, $str:expr, $( $e:expr ),+ ) => {{
            let s = format!( $str, $( $e ),+ );
            $crate::util::wasm::console_log(&s);
        }};
    }

    #[macro_export]
    macro_rules! dbg {
        ($fmt:literal) => {{
             out!("{}", $fmt);
        }};
        ($fmt:literal, $( $e:expr ),+ ) => {{
             out!($fmt, $( $e ),+ );
        }};
        ($expr:expr) => {{
            out!("{} = {:?}", stringify!($expr), $expr);
        }};
        () => {{
            out!("{}", "Nothing to see here");
        }};
    }

    #[macro_export]
    macro_rules! println {
        ($fmt:literal) => {{
             out!("{}", $fmt);
        }};
        ($fmt:literal, $( $e:expr ),+ ) => {{
             out!($fmt, $( $e ),+ );
        }};
        ($expr:expr) => {{
             out!("{} = {:?}", stringify!($expr), $expr);
        }};
        () => {{
            out!("{}", "Nothing to see here");
        }};
    }

    #[macro_export]
    macro_rules! print {
        ( $( $arg:tt )* ) => {{
            println!( $( $arg )* );
        }};
    }

    #[macro_export]
    macro_rules! panic {
        ( $( $arg:tt )* ) => {{
            println!( $( $arg )* );
            #[cfg(debug_assertions)]
            core::panic!();

            #[cfg(not(debug_assertions))]
            core::arch::wasm32::unreachable();
        }};
    }

    #[macro_export]
    macro_rules! unreachable {
        ( $( $arg:tt )* ) => {{
            panic!()
        }};
    }

    pub fn console_log(a: &str) {
        log(a);
    }

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        fn log(a: &str);
    }
}

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

// ----------------------------------------------------------------------------
//
//                                  POD ARRAY
//
// ----------------------------------------------------------------------------
use crate::alloc_api::*;
use alloc::alloc::Layout;
use core::ptr::NonNull;

struct DataInfo {
    size: usize,
    align: usize,
}

struct RawPod {
    data: NonNull<u8>,
    info: DataInfo,
    length: usize,
    capacity: usize,
}

impl RawPod {
    fn new(info: DataInfo) -> Self {
        // We use the same trick that std::vec::Vec uses
        return Self {
            data: NonNull::dangling(),
            info,
            length: 0,
            capacity: 0,
        };
    }

    fn realloc(&mut self, alloc: &dyn Allocator, capacity: usize) {
        let (size, align) = (self.info.size, self.info.align);
        let get_info = move |mut data: NonNull<[u8]>| -> (NonNull<u8>, usize) {
            let data = unsafe { data.as_mut() };
            let capacity = unwrap(data.len().checked_div(size));
            let data = unsafe { NonNull::new_unchecked(data.as_mut_ptr()) };

            return (data, capacity);
        };

        // We use the same trick that std::vec::Vec uses
        let (data, capacity) = match (size * self.capacity, size * capacity) {
            (x, y) if x == y => return,
            (0, 0) => {
                self.capacity = capacity;
                return;
            }

            (prev_size, 0) => {
                let layout = expect(Layout::from_size_align(prev_size, align));
                unsafe { alloc.deallocate(self.data, layout) };

                (NonNull::dangling(), capacity)
            }

            (0, new_size) => {
                let layout = expect(Layout::from_size_align(new_size, align));
                let data = expect(alloc.allocate(layout));

                get_info(data)
            }

            (prev_size, new_size) => {
                let prev_layout = expect(Layout::from_size_align(prev_size, align));
                let new_layout = expect(Layout::from_size_align(new_size, align));

                let result = unsafe {
                    if new_size > prev_size {
                        alloc.grow(self.data, prev_layout, new_layout)
                    } else {
                        alloc.shrink(self.data, prev_layout, new_layout)
                    }
                };

                let data = expect(result);

                get_info(data)
            }
        };

        self.data = data;
        self.length = core::cmp::min(self.length, capacity);
        self.capacity = capacity;
    }

    fn with_capacity(info: DataInfo, alloc: &dyn Allocator, capacity: usize) -> Self {
        // We use the same trick that std::vec::Vec uses
        let mut s = Self::new(info);
        s.realloc(alloc, capacity);

        return s;
    }
}

pub struct Pod<T, A>
where
    T: Copy,
    A: Allocator,
{
    raw: RawPod,
    phantom: core::marker::PhantomData<(T, A)>,
}

impl<T> Pod<T, Global>
where
    T: Copy,
{
    pub fn new() -> Self {
        let info = DataInfo {
            size: core::mem::size_of::<T>(),
            align: core::mem::align_of::<T>(),
        };

        return Self {
            raw: RawPod::new(info),
            phantom: core::marker::PhantomData,
        };
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut s = Self::new();
        s.raw.realloc(&Global, capacity);

        return s;
    }
}
