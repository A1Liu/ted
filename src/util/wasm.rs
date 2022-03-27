use super::*;

pub use wasm_bindgen::prelude::*;
pub use wasm_bindgen::JsCast;

pub type JsFunc = dyn 'static + FnMut() -> Result<(), JsValue>;

pub fn enclose(f: impl 'static + FnMut() -> Result<(), JsValue>) -> Closure<JsFunc> {
    return Closure::wrap(Box::new(f) as Box<JsFunc>);
}

pub fn get_canvas() -> Result<web_sys::HtmlCanvasElement, JsValue> {
    let err_map = || JsValue::from("RIP");
    let window = web_sys::window().ok_or_else(err_map)?;
    let document = window.document().ok_or_else(err_map)?;
    let canvas = document.get_element_by_id("canvas").ok_or_else(err_map)?;
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
        #[cfg(debug_assertions)]
        core::panic!( $( $arg )* );

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
