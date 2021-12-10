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

#[inline]
fn pt(x: u32, y: u32) -> Point {
    return Point { x, y };
}

pub fn render_text(canvas: web_sys::Element, text: &str) -> Result<(), JsValue> {
    let webgl = WebGl::new(canvas)?;

    let points: [Point; 3] = [pt(0, 0), pt(1, 0), pt(0, 1)];
    webgl.bind_array("in_pos", &points)?;

    // let glyphs: [f32; 1] = [1.0];

    let width: f32 = 2.0;
    webgl.bind_uniform("u_width", width)?;

    let height: f32 = 2.0;
    webgl.bind_uniform("u_height", height)?;

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
