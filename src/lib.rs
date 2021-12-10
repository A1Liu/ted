#![allow(dead_code)]
#![allow(unused_variables)]
// #![allow(unused_imports)]

pub mod fonts;
pub mod graphics;
pub mod large_text;
pub mod text;
pub mod util;
mod webgl;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use graphics::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(a: &str);
}

#[wasm_bindgen]
pub fn albert_editor_main_loop() {
    log("Hello World!\n");
}

#[wasm_bindgen]
pub fn render(canvas: web_sys::Element) -> Result<(), JsValue> {
    // let mut cache = GlyphCache::new();

    let text = "Hello World!";
    render_text(canvas, text)?;

    return Ok(());
}
