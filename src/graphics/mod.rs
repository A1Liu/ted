mod fonts;
mod webgl;

use fonts::*;
use wasm_bindgen::prelude::*;
use webgl::*;

pub struct TextVertices {
    points: Vec<Point>,
    glyphs: Vec<Glyph>,
    did_raster: bool,
    width: usize,
    height: usize,
}

pub fn render_text(ctx: web_sys::WebGl2RenderingContext, text: &str) -> Result<(), JsValue> {
    let mut webgl = WebGl::new(ctx)?;
    let mut cache = GlyphCache::new();

    let points = vec![pt(0, 0), pt(1, 0), pt(0, 1), pt(0, 1), pt(1, 0), pt(1, 1)];
    webgl.bind_array("in_pos", &points)?;

    let glyph_list = cache.translate_glyphs("e");
    webgl.bind_array("in_glyph_pos", &glyph_list.glyphs)?;

    let (atlas_width, atlas_height) = cache.atlas_dims();

    if glyph_list.did_raster {
        let atlas = cache.atlas();
        webgl.bind_texture("u_glyph_atlas", atlas_width, atlas_height, atlas)?;
    }

    let width: f32 = 2.0;
    webgl.bind_uniform("u_width", width)?;

    let height: f32 = 2.0;
    webgl.bind_uniform("u_height", height)?;

    webgl.bind_uniform("u_atlas_width", atlas_width)?;
    webgl.bind_uniform("u_atlas_height", atlas_height)?;

    webgl.draw(points.len() as i32);

    return Ok(());
}

#[derive(Clone, Copy)]
#[repr(C)]
struct Point {
    x: u32,
    y: u32,
}

#[inline]
fn pt(x: u32, y: u32) -> Point {
    return Point { x, y };
}

impl WebGlType for Point {
    const GL_TYPE: u32 = Context::UNSIGNED_INT;
    const SIZE: i32 = 2;

    unsafe fn view(array: &[Self]) -> js_sys::Object {
        let ptr = array.as_ptr() as *const u32;
        let buffer: &[u32] = core::slice::from_raw_parts(ptr, array.len() * 2);
        return js_sys::Uint32Array::view(buffer).into();
    }

    fn is_int() -> bool {
        return true;
    }
}

impl WebGlType for Glyph {
    const GL_TYPE: u32 = Context::UNSIGNED_INT;
    const SIZE: i32 = 2;

    unsafe fn view(array: &[Self]) -> js_sys::Object {
        let ptr = array.as_ptr() as *const u32;
        let buffer: &[u32] = core::slice::from_raw_parts(ptr, array.len() * 2);
        return js_sys::Uint32Array::view(buffer).into();
    }

    fn is_int() -> bool {
        return true;
    }
}
