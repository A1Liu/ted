mod fonts;
mod webgl;

pub use fonts::*;
pub use webgl::*;

use crate::util::*;
use wasm_bindgen::prelude::*;

pub struct TextShader {
    program: Program,

    // Vertices
    vao: VAO,

    // Uniform Locations
    u_glyph_atlas: ULoc,
    u_dims: ULoc,
    u_atlas_dims: ULoc,

    // Resources
    tex: Texture,
    in_pos: Buffer<Vector3<u32>>,
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
        let u_dims = gl.uloc(&program, "u_dims")?;
        let u_atlas_dims = gl.uloc(&program, "u_atlas_dims")?;

        let tex = gl.tex(&u_glyph_atlas, 0)?;

        return Ok(Self {
            program,
            vao,
            in_pos,
            in_glyph_pos,
            u_glyph_atlas,
            u_dims,
            u_atlas_dims,
            tex,
        });
    }

    pub fn render(
        &self,
        atlas: Option<&[u8]>,
        points: &[Vector3<u32>],
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

        let u_dims = Vector2 {
            x: dims.x as f32,
            y: dims.y as f32,
        };

        gl.bind_uniform(&self.u_dims, u_dims);

        let u_atlas_dims = Vector2 {
            x: atlas_dims.x as f32,
            y: atlas_dims.y as f32,
        };

        gl.bind_uniform(&self.u_atlas_dims, u_atlas_dims);

        gl.draw(points.len() as i32);

        return Ok(());
    }
}

thread_local! {
    pub static TEXT_SHADER: TextShader = expect(TextShader::new());
}
/*
pub struct TextVertices<'a> {
    cache: &'a mut GlyphCache,
    points: Vec<Vector3<u32>>,
    glyphs: Vec<Glyph>,
    did_raster: bool,
    dims: Rect,
    pos: Point2<u32>,
}

impl<'a> TextVertices<'a> {
    // TODO do calculations to determine what the actual dimensions should be
    // based on the canvas
    pub fn new(cache: &'a mut GlyphCache, dims: Rect, cursor_pos: Option<Point2<u32>>) -> Self {
        let glyph_list = cache.translate_glyphs(" ");
        let did_raster = glyph_list.did_raster;

        let size = (dims.x * dims.y) as usize;
        let mut glyphs = Vec::with_capacity(size * 6);
        let mut points = Vec::with_capacity(size * 6);

        for y in 0..dims.y {
            for x in 0..dims.x {
                points.extend_from_slice(&[
                    pt(x, y, 0),
                    pt(x + 1, y, 0),
                    pt(x, y + 1, 0),
                    pt(x, y + 1, 0),
                    pt(x + 1, y, 0),
                    pt(x + 1, y + 1, 0),
                ]);

                glyphs.extend_from_slice(&glyph_list.glyphs);
            }
        }

        // For z:
        // 0 is normal
        // 1 is cursor
        // 2 is selected
        // Set the block mode for the points that represent the cursor
        if let Some(pos) = cursor_pos {
            let idx = ((pos.y * dims.x + pos.x) * 6) as usize;

            for idx in idx..(idx + 6) {
                points[idx].z = 1;
            }
        }

        return Self {
            cache,
            points,
            glyphs,
            did_raster,
            dims,
            pos: Point2 { x: 0, y: 0 },
        };
    }

    pub fn push(&mut self, text: &str) -> bool {
        for c in text.chars() {
            if c == '\n' {
                if self.place_char(self.dims.x - self.pos.x, ' ') {
                    return true;
                }

                continue;
            }

            if c == '\t' {
                if self.place_char(2, ' ') {
                    return true;
                }

                continue;
            }

            if c.is_whitespace() {
                if self.place_char(1, ' ') {
                    return true;
                }

                continue;
            }

            if c.is_control() {
                continue;
            }

            if self.place_char(1, c) {
                return true;
            }
        }

        return false;
    }

    pub fn render(&mut self) -> Result<(), JsValue> {
        let atlas = self.did_raster.then(|| self.cache.atlas());
        let atlas_dims = self.cache.atlas_dims();


        return Ok(());
    }
}
*/

// For z:
// 0 is normal
// 1 is cursor
// 2 is selected

#[inline]
fn pt(x: u32, y: u32, block_type: u32) -> Vector3<u32> {
    let z = block_type;
    return Vector3 { x, y, z };
}

#[inline]
fn cursor(x: u32, y: u32) -> Vector3<u32> {
    return Vector3 { x, y, z: 1 };
}

type TextSlot = Vector3<u32>;
