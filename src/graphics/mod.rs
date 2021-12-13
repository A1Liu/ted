mod fonts;
mod webgl;

use crate::util::*;
pub use fonts::*;
use mint::Point2;
use wasm_bindgen::prelude::*;
pub use webgl::*;

struct TextShader {
    program: Program,

    // Vertices
    vao: VAO,

    // Uniform Locations
    u_glyph_atlas: ULoc,
    u_width: ULoc,
    u_height: ULoc,
    u_atlas_width: ULoc,
    u_atlas_height: ULoc,

    // Resources
    tex: Texture,
    in_pos: Buffer<Point2<u32>>,
    in_glyph_pos: Buffer<Glyph>,
}

impl TextShader {
    fn new() -> Result<Self, JsValue> {
        let vert_text = core::include_str!("./vertex.glsl");
        let frag_text = core::include_str!("./fragment.glsl");
        let program = gl.compile(vert_text, frag_text)?;

        let vao = gl.vao()?;
        let in_pos = gl.attr_buffer(&program, "in_pos")?;
        let in_glyph_pos = gl.attr_buffer(&program, "in_glyph_pos")?;
        let u_glyph_atlas = gl.uloc(&program, "u_glyph_atlas")?;
        let u_width = gl.uloc(&program, "u_width")?;
        let u_height = gl.uloc(&program, "u_height")?;
        let u_atlas_width = gl.uloc(&program, "u_atlas_width")?;
        let u_atlas_height = gl.uloc(&program, "u_atlas_height")?;

        let tex = gl.tex(&u_glyph_atlas, 0)?;

        return Ok(Self {
            program,
            vao,
            in_pos,
            in_glyph_pos,
            u_glyph_atlas,
            u_width,
            u_height,
            u_atlas_width,
            u_atlas_height,
            tex,
        });
    }

    fn render(
        &self,
        atlas: Option<&[u8]>,
        points: &[Point2<u32>],
        glyphs: &[Glyph],
        atlas_dims: Rect,
        dims: Rect,
    ) -> Result<(), JsValue> {
        gl.use_program(&self.program);

        gl.write_buffer(&self.in_pos, points);
        gl.write_buffer(&self.in_glyph_pos, glyphs);
        if let Some(atlas) = atlas {
            gl.update_tex(&self.tex, atlas_dims, atlas)?;
        }

        gl.bind_vao(&self.vao);
        gl.bind_tex(&self.u_glyph_atlas, 0, &self.tex);
        gl.bind_uniform(&self.u_width, dims.width as f32);
        gl.bind_uniform(&self.u_height, dims.height as f32);
        gl.bind_uniform(&self.u_atlas_width, atlas_dims.width);
        gl.bind_uniform(&self.u_atlas_height, atlas_dims.height);

        gl.draw(points.len() as i32);

        return Ok(());
    }
}

thread_local! {
    static TEXT_SHADER: TextShader = TextShader::new().unwrap();
}

pub struct TextVertices<'a> {
    cache: &'a mut GlyphCache,
    points: Vec<Point2<u32>>,
    glyphs: Vec<Glyph>,
    did_raster: bool,
    dims: Rect,
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
            dims: Rect::new(width, height),
            current_line: 0,
            current_column: 0,
        };
    }

    fn place_char(&mut self, len: u32) -> (bool, Option<(u32, u32)>) {
        let (width, height) = (self.dims.width, self.dims.height);

        loop {
            if self.current_line >= height {
                return (true, None);
            }

            let col = self.current_column;
            let end = col + len;
            if end < width {
                self.current_column += len;
                return (false, Some((self.current_line, col)));
            }

            self.current_column = 0;
            let line = self.current_line;
            self.current_line += 1;

            if col == 0 || end == width {
                return (self.current_line >= height, Some((line, col)));
            }
        }
    }

    pub fn push(&mut self, text: &str) -> bool {
        let (width, height) = (self.dims.width, self.dims.height);
        dbg!();

        for c in text.chars() {
            if c == '\n' {
                let (none_left, place) = self.place_char(width - self.current_column);
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

            dbg!(c);

            let (none_left, place) = self.place_char(1);
            let (line, col) = match place {
                Some(loc) => loc,
                None => return true,
            };

            dbg!(c);

            let (x, y) = (col, line);

            self.points.extend_from_slice(&[
                pt(x, y),
                pt(x + 1, y),
                pt(x, y + 1),
                pt(x, y + 1),
                pt(x + 1, y),
                pt(x + 1, y + 1),
            ]);

            dbg!(c);

            let mut tmp = [0; 4];
            let c_str = c.encode_utf8(&mut tmp);

            dbg!(c);

            let glyph_list = self.cache.translate_glyphs(c_str);
            self.did_raster = self.did_raster || glyph_list.did_raster;
            self.glyphs.extend(glyph_list.glyphs);

            dbg!(c);

            if none_left {
                return true;
            }
        }

        return false;
    }

    pub fn render(&self) -> Result<(), JsValue> {
        let atlas = match self.did_raster {
            true => Some(self.cache.atlas()),
            false => None,
        };

        let atlas_dims = self.cache.atlas_dims();
        let dims = self.dims;

        TEXT_SHADER.with(move |shader| -> Result<(), JsValue> {
            shader.render(atlas, &self.points, &self.glyphs, atlas_dims, dims)?;

            return Ok(());
        })?;

        return Ok(());
    }
}

#[inline]
fn pt(x: u32, y: u32) -> Point2<u32> {
    return Point2 { x, y };
}

impl WebGlType for Point2<u32> {
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
