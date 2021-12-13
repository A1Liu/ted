use wasm_bindgen::prelude::*;

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
        $crate::print_utils::console_log(&s);
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
        core::arch::wasm32::unreachable();
    }};
}

#[macro_export]
macro_rules! unreachable {
    ( $( $arg:tt )* ) => {{
        println!( $( $arg )* );
        core::arch::wasm32::unreachable();
    }};
}

pub fn console_log(a: &str) {
    log(a);
}

pub fn console_warn(a: &str) {
    log(a);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(a: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn warn(a: &str);
}
