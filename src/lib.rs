// Long-term
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_macros)]
#![allow(unused_braces)]
#![allow(non_upper_case_globals)]
// Short-term allows
/* */
#![allow(unused_imports)]
#![allow(unused_mut)]
/* */

// #[macro_use]
// extern crate lazy_static;

#[cfg(target_arch = "wasm32")]
#[macro_use]
mod print_utils;

mod btree;
mod text;
mod util;

#[cfg(target_arch = "wasm32")]
mod window;

#[cfg(target_arch = "wasm32")]
pub use wasm_exports::*;

#[cfg(target_arch = "wasm32")]
mod graphics;

#[cfg(target_arch = "wasm32")]
mod wasm_exports {
    use crate::graphics::*;
    use crate::util::*;
    use crate::window::*;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    fn enclose(
        f: impl 'static + FnMut() -> Result<(), JsValue>,
    ) -> Closure<dyn 'static + FnMut() -> Result<(), JsValue>> {
        return Closure::wrap(Box::new(f) as Box<dyn FnMut() -> Result<(), JsValue>>);
    }

    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

    #[wasm_bindgen]
    pub fn start() {
        #[cfg(debug_assertions)]
        {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        }

        let f = enclose(move || start_window());
        prevent_throw(&f);
    }

    #[wasm_bindgen(inline_js = r#"
export const repeat = async (func, ms = 1000, limit = 100) => {
  while (limit-- > 0) {
    func();
    await new Promise((res) => setTimeout(res, ms));
  }
};

export const preventThrow = (fn) => {
  try {
    fn();
  } catch (e) {}
};"#)]
    extern "C" {
        #[wasm_bindgen(js_name = "preventThrow")]
        fn prevent_throw(func: &Closure<dyn FnMut() -> Result<(), JsValue>>);

        fn repeat(func: &Closure<dyn FnMut() -> Result<(), JsValue>>);
    }
}
