mod fonts;
mod webgl;

pub use fonts::*;
use wasm_bindgen::prelude::*;
pub use webgl::*;

pub struct TextVertices<'a> {
    cache: &'a mut GlyphCache,
    points: Vec<Point>,
    glyphs: Vec<Glyph>,
    did_raster: bool,
    width: u32,
    height: u32,
    current_line: u32,
    current_column: u32,
}

impl<'a> TextVertices<'a> {
    // TODO do calculations to determine what the actual dimensions should be
    // based on the canvas
    pub fn new(cache: &'a mut GlyphCache, width: u32, height: u32) -> Self {
        return Self {
            cache,
            points: Vec::new(),
            glyphs: Vec::new(),
            did_raster: false,
            width,
            height,
            current_line: 0,
            current_column: 0,
        };
    }

    fn place_char(&mut self, len: u32) -> (bool, Option<(u32, u32)>) {
        loop {
            if self.current_line >= self.height {
                return (true, None);
            }

            let col = self.current_column;
            let end = col + len;
            if end < self.width {
                self.current_column += len;
                return (false, Some((self.current_line, col)));
            }

            self.current_column = 0;
            let line = self.current_line;
            self.current_line += 1;

            if col == 0 || end == self.width {
                return (self.current_line >= self.height, Some((line, col)));
            }
        }
    }

    pub fn push(&mut self, text: &str) -> bool {
        for c in text.chars() {
            if c == '\n' {
                let (none_left, place) = self.place_char(self.width - self.current_column);
                if none_left || place.is_none() {
                    return true;
                }

                continue;
            }

            if c == '\t' {
                let (none_left, place) = self.place_char(2);
                if none_left || place.is_none() {
                    return true;
                }

                continue;
            }

            if c.is_whitespace() {
                let (none_left, place) = self.place_char(1);
                if none_left || place.is_none() {
                    return true;
                }

                continue;
            }

            if c.is_control() {
                continue;
            }

            let (none_left, place) = self.place_char(1);
            let (line, col) = match place {
                Some(loc) => loc,
                None => return true,
            };

            let (x, y) = (col, line);

            self.points.extend_from_slice(&[
                pt(x, y),
                pt(x + 1, y),
                pt(x, y + 1),
                pt(x, y + 1),
                pt(x + 1, y),
                pt(x + 1, y + 1),
            ]);

            let mut tmp = [0; 4];
            let c_str = c.encode_utf8(&mut tmp);

            let glyph_list = self.cache.translate_glyphs(c_str);
            self.did_raster = self.did_raster || glyph_list.did_raster;
            self.glyphs.extend(glyph_list.glyphs);

            if none_left {
                return true;
            }
        }

        return false;
    }

    pub fn render(&self, webgl: &mut WebGl) -> Result<(), JsValue> {
        webgl.bind_array("in_pos", &self.points)?;
        webgl.bind_array("in_glyph_pos", &self.glyphs)?;

        let (atlas_width, atlas_height) = self.cache.atlas_dims();

        if self.did_raster {
            let atlas = self.cache.atlas();
            webgl.bind_texture("u_glyph_atlas", atlas_width, atlas_height, atlas)?;
        }

        let width: f32 = self.width as f32;
        webgl.bind_uniform("u_width", width)?;

        let height: f32 = self.height as f32;
        webgl.bind_uniform("u_height", height)?;

        webgl.bind_uniform("u_atlas_width", atlas_width)?;
        webgl.bind_uniform("u_atlas_height", atlas_height)?;

        webgl.draw(self.points.len() as i32);

        return Ok(());
    }
}

pub fn render_text(webgl: &mut WebGl, text: &str) -> Result<(), JsValue> {
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
