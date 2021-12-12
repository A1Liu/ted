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

    pub fn render(&self, webgl: &WebGl) -> Result<(), JsValue> {
        let vert_text = core::include_str!("./vertex.glsl");
        let frag_text = core::include_str!("./fragment.glsl");
        let program = webgl.compile(vert_text, frag_text)?;
        webgl.use_program(&program);

        let in_pos = webgl.vloc(&program, "in_pos")?;
        webgl.bind_array(in_pos, &self.points)?;

        let in_glyph_pos = webgl.vloc(&program, "in_glyph_pos")?;
        webgl.bind_array(in_glyph_pos, &self.glyphs)?;

        let atlas_dims = self.cache.atlas_dims();

        if self.did_raster {
            let atlas = self.cache.atlas();
            let u_glyph_atlas = webgl.uloc(&program, "u_glyph_atlas")?;
            webgl.bind_tex(u_glyph_atlas, 0, atlas_dims, atlas)?;
        }

        let u_width = webgl.uloc(&program, "u_width")?;
        webgl.bind_uniform(u_width, self.width as f32)?;

        let u_height = webgl.uloc(&program, "u_height")?;
        webgl.bind_uniform(u_height, self.height as f32)?;

        let u_atlas_width = webgl.uloc(&program, "u_atlas_width")?;
        webgl.bind_uniform(u_atlas_width, atlas_dims.width)?;

        let u_atlas_height = webgl.uloc(&program, "u_atlas_height")?;
        webgl.bind_uniform(u_atlas_height, atlas_dims.height)?;

        webgl.draw(self.points.len() as i32);

        return Ok(());
    }
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
