// Long-term
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_macros)]
// Short-term allows
/* */
#![allow(unused_imports)]
#![allow(unused_mut)]
/* */

#[cfg(target_arch = "wasm32")]
#[macro_use]
mod print_utils;

mod btree;
mod graphics;
mod text;
mod util;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use graphics::*;
use util::*;

#[wasm_bindgen]
pub fn test_print() {
    println!("Hello World!");
}

#[wasm_bindgen]
pub fn text() {
    let mut tree = btree::BTree::new();
    for i in 0..8 {
        println!("iter: {}", i);
        tree.insert(0, text::BufferView::new());
    }
}

#[wasm_bindgen]
pub fn render(ctx: web_sys::WebGl2RenderingContext) -> Result<(), JsValue> {
    let text = "Hello World!";
    render_text(ctx, text)?;

    return Ok(());
}
