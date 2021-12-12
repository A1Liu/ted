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

#[macro_use]
extern crate lazy_static;

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

#[wasm_bindgen(js_name = "newWebgl")]
pub fn new_webgl() -> Result<graphics::WebGl, JsValue> {
    return graphics::WebGl::new();
}

#[wasm_bindgen]
pub fn render(webgl: &graphics::WebGl) -> Result<(), JsValue> {
    let mut cache = GlyphCache::new();

    let mut file = text::File::new();

    let text = "Hello World!\n\nWelcome to my stupid project to make a text editor.";
    file.insert(0, text);

    let mut vertices = TextVertices::new(&mut cache, 28, 10);

    for text in &file {
        vertices.push(text);
    }

    gl.with(move |ctx| vertices.render(ctx))?;

    return Ok(());
}
