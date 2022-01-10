mod fonts;
mod webgl;

use crate::highlighting::*;
use crate::util::*;
use mint::Vector3;

pub use fonts::*;
pub use webgl::*;

pub struct TextShader {
    program: Program,

    // Vertices
    vao: VAO,

    // Uniform Locations
    u_dims: ULoc,
    u_clip_begin: ULoc,
    u_clip_end: ULoc,
    u_atlas_dims: ULoc,
    u_glyph_atlas: ULoc,

    // Resources
    tex: Texture,
    in_fg_color: Buffer<Color>,
    in_bg_color: Buffer<Color>,
    in_glyph_pos: Buffer<Glyph>,
}

pub struct TextShaderInput<'a> {
    pub is_lines: bool,
    pub atlas: Option<&'a [u8]>,
    pub fg_colors: Vec<Color>,
    pub bg_colors: Vec<Color>,
    pub glyphs: Vec<Glyph>,
    pub atlas_dims: Rect,
    pub dims: Rect,
}

impl TextShader {
    fn new() -> Result<Self, JsValue> {
        let vert_text = core::include_str!("./vertex.glsl");
        let frag_text = core::include_str!("./fragment.glsl");
        let program = gl.compile(vert_text, frag_text)?;

        let vao = gl.vao()?;

        let in_fg_color = gl.attr_buffer(&program, "in_fg_color")?;
        let in_bg_color = gl.attr_buffer(&program, "in_bg_color")?;
        let in_glyph_pos = gl.attr_buffer(&program, "in_glyph_pos")?;

        let u_dims = gl.uloc(&program, "u_dims")?;
        let u_clip_begin = gl.uloc(&program, "u_clip_begin")?;
        let u_clip_end = gl.uloc(&program, "u_clip_end")?;
        let u_atlas_dims = gl.uloc(&program, "u_atlas_dims")?;
        let u_glyph_atlas = gl.uloc(&program, "u_glyph_atlas")?;

        let tex = gl.tex(&u_glyph_atlas, 0)?;

        return Ok(Self {
            program,
            vao,

            in_fg_color,
            in_bg_color,
            in_glyph_pos,

            u_glyph_atlas,
            u_clip_begin,
            u_clip_end,
            u_dims,
            u_atlas_dims,

            tex,
        });
    }

    pub fn render(&self, input: TextShaderInput) -> Result<(), JsValue> {
        gl.use_program(&self.program);

        gl.write_buffer(&self.in_fg_color, &input.fg_colors);
        gl.write_buffer(&self.in_bg_color, &input.bg_colors);
        gl.write_buffer(&self.in_glyph_pos, &input.glyphs);
        if let Some(atlas) = input.atlas {
            gl.update_tex(&self.tex, input.atlas_dims, &atlas)?;
        }

        gl.bind_vao(&self.vao);
        gl.bind_tex(&self.u_glyph_atlas, 0, &self.tex);

        let u_dims = Vector2 {
            x: input.dims.x as f32,
            y: input.dims.y as f32,
        };

        gl.bind_uniform(&self.u_dims, u_dims);

        let u_atlas_dims = Vector2 {
            x: input.atlas_dims.x as f32,
            y: input.atlas_dims.y as f32,
        };

        gl.bind_uniform(&self.u_atlas_dims, u_atlas_dims);

        // TODO(HACK): Line numbers require space on the left-hand side. Instead
        // of actually calculating how much space we need, we will just use
        // whatever random number looks ok for now. Eventually we should replace
        // 'is_lines' with actual offsets, likely in clip-space units
        //                              - Albert Liu, Dec 24, 2021 Fri 15:31 EST
        let (begin, end) = match input.is_lines {
            true => (-1.0f32, -0.9f32),
            false => (-0.9f32, 1.0f32),
        };

        gl.bind_uniform(&self.u_clip_begin, begin);
        gl.bind_uniform(&self.u_clip_end, end);

        gl.draw((input.dims.x * input.dims.y * 6) as i32);

        return Ok(());
    }
}

thread_local! {
    pub static TEXT_SHADER: TextShader = expect(TextShader::new());
}

impl WebGlType for Color {
    const GL_TYPE: u32 = Context::FLOAT;
    const SIZE: i32 = 3;

    unsafe fn view(array: &[Self]) -> js_sys::Object {
        let ptr = array.as_ptr() as *const f32;
        let buffer: &[f32] = core::slice::from_raw_parts(ptr, array.len() * 3);
        return js_sys::Float32Array::view(buffer).into();
    }
}
