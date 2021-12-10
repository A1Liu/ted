use crate::webgl::*;
use wasm_bindgen::prelude::*;
use web_sys::WebGlRenderingContext;

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
    x: f32,
    y: f32,
}

#[inline]
fn pt(x: f32, y: f32) -> Point {
    return Point { x, y };
}

pub fn render_text(canvas: web_sys::Element, text: &str) -> Result<(), JsValue> {
    let webgl = WebGl::new(canvas)?;

    let points: [Point; 3] = [pt(0.0, 0.0), pt(1.0, 0.0), pt(0.0, 1.0)];
    webgl.bind_array("a_pos", &points)?;

    // let glyphs: [f32; 1] = [1.0];

    let width: f32 = 2.0;
    webgl.bind_uniform("width", width)?;

    let height: f32 = 2.0;
    webgl.bind_uniform("height", height)?;

    webgl.draw(points.len() as i32);

    return Ok(());
}

impl WebGlType for Point {
    const GL_TYPE: u32 = WebGlRenderingContext::FLOAT;
    const SIZE: i32 = 2;

    unsafe fn view(array: &[Self]) -> js_sys::Object {
        let ptr = array.as_ptr() as *const f32;
        let buffer: &[f32] = core::slice::from_raw_parts(ptr, array.len() * 2);
        return js_sys::Float32Array::view(buffer).into();
    }
}
