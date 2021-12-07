#![allow(dead_code)]
#![allow(unused_variables)]

mod graphics;
mod large_text;
mod text;
mod util;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use wasm_bindgen::prelude::*;

#[rustfmt::skip] // rustfmt deletes the keyword async
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(a: &str);
}

#[wasm_bindgen]
pub fn albert_editor_main_loop() {
    log("Hello World!\n");
}
