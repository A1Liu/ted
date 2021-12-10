use crate::webgl::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};

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

#[repr(C, packed)]
pub struct Point {
    x: u32,
    y: u32,
}

fn pt(x: u32, y: u32) -> Point {
    return Point { x, y };
}

pub fn render_text(canvas: web_sys::Element, text: &str) -> Result<(), JsValue> {
    let webgl = WebGl::new(canvas)?;

    let points: [Point; 4] = [pt(0, 0), pt(0, 1), pt(1, 0), pt(1, 1)];
    webgl.bind_array("position", &points)?;

    return Ok(());
}

impl WebGlType for Point {
    const GL_TYPE: u32 = WebGlRenderingContext::UNSIGNED_INT;
    const SIZE: i32 = 2;

    unsafe fn view(array: &[Self]) -> js_sys::Object {
        let ptr = array.as_ptr() as *const u32;
        let buffer: &[u32] = core::slice::from_raw_parts(ptr, array.len() * 2);
        return js_sys::Uint32Array::view(buffer).into();
    }
}
