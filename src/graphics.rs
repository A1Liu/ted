use crate::fonts::*;
use crate::util::*;
use crate::webgl::*;
use wasm_bindgen::prelude::*;

pub enum SourceId {
    File { id: usize },
    Text { id: usize },
}

pub struct Canvas {
    pub source: SourceId,
    pub screen_id: usize,
    pub width: usize,
    pub height: usize,
}

pub struct Graphics {
    screens: Vec<Canvas>,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Point {
    x: u32,
    y: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct GlyphData {
    value: [u32; 3],
}

#[inline]
fn pt(x: u32, y: u32) -> Point {
    return Point { x, y };
}

// #[inline]
// fn glyph(offset: u32) -> GlyphData {
//     return GlyphData { value: [] };
// }

pub fn render_text(canvas: web_sys::Element, text: &str) -> Result<(), JsValue> {
    let mut webgl = WebGl::new(canvas)?;
    let mut cache = GlyphCache::new();

    let loc = webgl
        .ctx
        .get_attrib_location(&webgl.program, "in_glyph_pos");
    if loc < 0 {
        return Err(JsValue::from("What's going on?"));
    }

    let points = vec![pt(0, 0), pt(1, 0), pt(0, 1), pt(0, 1), pt(1, 0), pt(1, 1)];
    webgl.bind_array("in_pos", &points)?;

    let glyph_list = cache.translate_glyphs("H");
    webgl.bind_array("in_glyph_pos", &glyph_list.glyphs)?;

    let (atlas_width, atlas_height) = cache.atlas_dims();

    if glyph_list.did_raster {
        console_log("Ho there");
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
