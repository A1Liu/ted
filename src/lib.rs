#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

pub mod fonts;
pub mod graphics;
pub mod large_text;
pub mod text;
pub mod util;
mod webgl;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use fonts::GlyphCache;
use graphics::*;
use wasm_bindgen::prelude::*;
use web_sys::WebGlRenderingContext;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(a: &str);
}

#[wasm_bindgen]
pub fn albert_editor_main_loop() {
    log("Hello World!\n");
}

/*
#[wasm_bindgen]
pub fn render(canvas: web_sys::Element) -> Result<(), JsValue> {
    let context = get_context(canvas)?;
    let mut cache = GlyphCache::new();

    let text = "Hello World!";
    let result = cache.translate_glyphs(text);
    let glyphs = result.glyphs;

    let vert_shader = compile_shader(
        &context,
        ShaderType::Vertex,
        core::include_str!("./vertex.glsl"),
    )?;

    let frag_shader = compile_shader(
        &context,
        ShaderType::Fragment,
        core::include_str!("./fragment.glsl"),
    )?;

    let program = link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));

    let vertices: [f32; 9] = [-0.7, -0.7, 0.0, 0.7, -0.7, 0.0, 0.0, 0.7, 0.0];
    vertex_buffer(&context, &program, &vertices, "position")?;

    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    context.draw_arrays(
        WebGlRenderingContext::TRIANGLES,
        0,
        (vertices.len() / 3) as i32,
    );

    Ok(())
}
*/
