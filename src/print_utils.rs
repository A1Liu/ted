use wasm_bindgen::prelude::*;

#[macro_export]
macro_rules! out {
    (@DEBUG, $str:expr, $( $e:expr ),+ ) => {{
        #[cfg(debug_assertions)]
        {
            out!(@CLEAN, core::concat!("DEBUG ({}:{}): ", $str, "\n"), file!(), line!(), $( $e ),+ );
        }
    }};
    (@LOG, $str:expr, $( $e:expr ),+ ) => {{
        out!(@CLEAN, core::concat!("LOG ({}:{}): ", $str, "\n"), file!(), line!(), $( $e ),+ );
    }};
    (@CLEAN, $str:expr, $( $e:expr ),+ ) => {{
        let s = format!( $str, $( $e ),+ );
        $crate::print_utils::console_log(&s);
    }};
}

#[macro_export]
macro_rules! dbg {
    ($fmt:literal) => {{
         out!(@DEBUG, "{}", $fmt);
    }};
    ($fmt:literal, $( $e:expr ),+ ) => {{
         out!(@DEBUG, $fmt, $( $e ),+ );
    }};
    ($expr:expr) => {{
         out!(@DEBUG, "{} = {:?}", stringify!($expr), $expr);
    }};
    () => {{
        out!(@DEBUG, "Nothing to see here");
    }};
}

#[macro_export]
macro_rules! panic {
    ( $( $arg:tt )* ) => {{
        dbg!( $( $arg )* );
        core::panic!();
    }};
}

#[macro_export]
macro_rules! println {
    ($fmt:literal) => {{
         out!(@LOG, "{}", $fmt);
    }};
    ($fmt:literal, $( $e:expr ),+ ) => {{
         out!(@LOG, $fmt, $( $e ),+ );
    }};
    ($expr:expr) => {{
         out!(@LOG, "{} = {:?}", stringify!($expr), $expr);
    }};
    () => {{
        out!(@LOG, "Nothing to see here");
    }};
}

#[macro_export]
macro_rules! print {
    ( $( $arg:tt )* ) => {{
        println!( $( $arg )* );
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
