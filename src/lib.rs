// Long-term
#![allow(dead_code)]
#![allow(unused_variables)]
// Short-term allows
/* */
#![allow(unused_imports)]
#![allow(unused_mut)]
/* */

mod btree;
mod webgl;

pub mod fonts;
pub mod graphics;
pub mod large_text;
pub mod text;
pub mod util;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub use btree::*;
use graphics::*;
use util::*;

#[wasm_bindgen]
pub fn albert_editor_main_loop() {
    console_log("Hello World!\n");
}

#[wasm_bindgen]
pub fn text() {
    let mut tree = btree::BTree::new();
    for i in 0..8 {
        println!("iter: {}", i);
        tree.insert(0, large_text::BufferView::new());
    }
}

#[wasm_bindgen]
pub fn render(canvas: web_sys::Element) -> Result<(), JsValue> {
    let text = "Hello World!";
    render_text(canvas, text)?;

    return Ok(());
}
