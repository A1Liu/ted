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
pub fn render(ctx: web_sys::WebGl2RenderingContext) -> Result<(), JsValue> {
    let mut webgl = WebGl::new(ctx)?;
    let mut cache = GlyphCache::new();

    let mut file = text::File::new();

    let text = "Hello World!\n\nWelcome to my stupid project to make a text editor.";
    file.insert(0, text);

    let mut vertices = TextVertices::new(&mut cache, 28, 10);

    for text in &file {
        vertices.push(text);
    }

    vertices.render(&mut webgl)?;

    return Ok(());
}
