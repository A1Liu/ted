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

    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

    #[wasm_bindgen]
    pub fn start() {
        #[cfg(debug_assertions)]
        {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        }

        start_window();
    }

    #[wasm_bindgen]
    pub fn render(s: &str) -> Result<(), JsValue> {
        let mut cache = GlyphCache::new();
        let mut vertices = TextVertices::new(&mut cache, 28, 15);

        vertices.push(s);

        vertices.render()?;

        return Ok(());
    }
}
