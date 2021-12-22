mod fonts;
mod webgl;

pub use fonts::*;
pub use webgl::*;

use crate::util::*;

pub struct TextShader {
    program: Program,

    // Vertices
    vao: VAO,

    // Uniform Locations
    u_dims: ULoc,
    u_atlas_dims: ULoc,
    u_glyph_atlas: ULoc,

    // Resources
    tex: Texture,
    in_pos: Buffer<CharBox>,
    in_block_type: Buffer<BlockType>,
    in_glyph_pos: Buffer<Glyph>,
}

impl TextShader {
    fn new() -> Result<Self, JsValue> {
        let vert_text = core::include_str!("./vertex.glsl");
        let frag_text = core::include_str!("./fragment.glsl");
        let program = gl.compile(vert_text, frag_text)?;

        let vao = gl.vao()?;

        let in_pos = gl.attr_buffer(&program, "in_pos")?;
        let in_block_type = gl.attr_buffer(&program, "in_block_type")?;
        let in_glyph_pos = gl.attr_buffer(&program, "in_glyph_pos")?;

        let u_dims = gl.uloc(&program, "u_dims")?;
        let u_atlas_dims = gl.uloc(&program, "u_atlas_dims")?;
        let u_glyph_atlas = gl.uloc(&program, "u_glyph_atlas")?;

        let tex = gl.tex(&u_glyph_atlas, 0)?;

        return Ok(Self {
            program,
            vao,

            in_pos,
            in_block_type,
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
        points: &[CharBox],
        block_types: &[BlockType],
        glyphs: &[Glyph],
        atlas_dims: Rect,
        dims: Rect,
    ) -> Result<(), JsValue> {
        gl.use_program(&self.program);

        gl.write_buffer(&self.in_pos, points);
        gl.write_buffer(&self.in_block_type, block_types);
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

        gl.draw((points.len() * 6) as i32);

        return Ok(());
    }
}

thread_local! {
    pub static TEXT_SHADER: TextShader = expect(TextShader::new());
}

#[derive(Clone, Copy)]
pub struct CharBox {
    // each box is 2 trianges of 3 points each
    top_left_1: Point2<u32>,
    top_right_2: Point2<u32>,
    bot_left_3: Point2<u32>,
    bot_left_4: Point2<u32>,
    top_right_5: Point2<u32>,
    bot_right_6: Point2<u32>,
}

#[inline]
pub fn pt(x: u32, y: u32) -> CharBox {
    let pt = |x: u32, y: u32| Point2 { x, y };

    return CharBox {
        top_left_1: pt(x, y),
        top_right_2: pt(x + 1, y),
        bot_left_3: pt(x, y + 1),
        bot_left_4: pt(x, y + 1),
        top_right_5: pt(x + 1, y),
        bot_right_6: pt(x + 1, y + 1),
    };
}

impl WebGlType for CharBox {
    const GL_TYPE: u32 = Context::UNSIGNED_INT;
    const SIZE: i32 = 2;

    unsafe fn view(array: &[Self]) -> js_sys::Object {
        let ptr = array.as_ptr() as *const u32;
        let buffer: &[u32] = core::slice::from_raw_parts(ptr, array.len() * 2 * 6);
        return js_sys::Uint32Array::view(buffer).into();
    }

    fn is_int() -> bool {
        return true;
    }
}

// For z:
// 0 is normal
// 1 is cursor
// 2 is selected
#[derive(Clone, Copy)]
#[repr(C)]
pub struct BlockType {
    // each box is 2 trianges of 3 points each
    top_left_1: u32,
    top_right_2: u32,
    bot_left_3: u32,
    bot_left_4: u32,
    top_right_5: u32,
    bot_right_6: u32,
}

impl BlockType {
    pub const Normal: Self = Self::new(0);
    pub const Cursor: Self = Self::new(1);

    const fn new(value: u32) -> Self {
        return Self {
            top_left_1: value,
            top_right_2: value,
            bot_left_3: value,
            bot_left_4: value,
            top_right_5: value,
            bot_right_6: value,
        };
    }
}

impl WebGlType for BlockType {
    const GL_TYPE: u32 = Context::UNSIGNED_INT;
    const SIZE: i32 = 1;

    unsafe fn view(array: &[Self]) -> js_sys::Object {
        let ptr = array.as_ptr() as *const u32;
        let buffer: &[u32] = core::slice::from_raw_parts(ptr, array.len() * 6);
        return js_sys::Uint32Array::view(buffer).into();
    }

    fn is_int() -> bool {
        return true;
    }
}
