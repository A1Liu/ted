use crate::util::*;
use js_sys::Object;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
pub use web_sys::WebGl2RenderingContext as Context;
pub use web_sys::WebGlUniformLocation as ULoc;
use web_sys::{WebGlProgram, WebGlShader};

#[repr(u32)]
pub enum ShaderType {
    Vertex = Context::VERTEX_SHADER,
    Fragment = Context::FRAGMENT_SHADER,
}

#[repr(u32)]
pub enum BufferKind {
    Array = Context::ARRAY_BUFFER,
}

#[repr(u32)]
pub enum UsagePattern {
    Static = Context::STATIC_DRAW,
}

pub trait WebGlType
where
    Self: Sized + Copy,
{
    const GL_TYPE: u32;
    const SIZE: i32;

    unsafe fn view(array: &[Self]) -> Object;

    fn is_int() -> bool {
        return false;
    }

    fn bind_uniform(self, ctx: &Context, loc: Option<&web_sys::WebGlUniformLocation>) {
        unimplemented!();
    }
}

#[derive(Clone, Copy)]
pub struct VLoc(u32);

// Maybe later make this more convenient to use with multiple programs
#[wasm_bindgen]
pub struct WebGl {
    ctx: Context,
}

pub struct Program {
    program: WebGlProgram,
    textures_used: u32,
}

impl WebGl {
    pub fn new() -> Result<Self, JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

        let options = webgl_context();

        let ctx = canvas
            .get_context_with_context_options("webgl2", &options)?
            .unwrap()
            .dyn_into::<Context>()?;

        return Ok(WebGl { ctx });
    }

    pub fn use_program(&self, prog: &Program) {
        self.ctx.use_program(Some(&prog.program));
    }

    pub fn compile(&self, vert: &str, frag: &str) -> Result<Program, JsValue> {
        let vert_shader = compile_shader(&self.ctx, ShaderType::Vertex, vert)?;
        let frag_shader = compile_shader(&self.ctx, ShaderType::Fragment, frag)?;
        let program = link_program(&self.ctx, &vert_shader, &frag_shader)?;

        return Ok(Program {
            program,
            textures_used: 0,
        });
    }

    pub fn draw(&self, triangles: i32) {
        self.ctx.clear_color(0.0, 0.0, 0.0, 1.0);
        self.ctx.clear(Context::COLOR_BUFFER_BIT);
        self.ctx.draw_arrays(Context::TRIANGLES, 0, triangles);
    }

    pub fn vloc(&self, prog: &Program, name: &str) -> Result<VLoc, JsValue> {
        let loc = self.ctx.get_attrib_location(&prog.program, name);
        let make_err = |e| format!("failed to get location of '{}' (got {})", name, loc);
        let loc = loc.try_into().map_err(make_err)?;

        return Ok(VLoc(loc));
    }

    pub fn uloc(&self, prog: &Program, name: &str) -> Result<ULoc, JsValue> {
        let make_err = || format!("Failed to write uniform");
        let loc_opt = self.ctx.get_uniform_location(&prog.program, name);
        let loc = loc_opt.ok_or_else(make_err)?;

        return Ok(loc);
    }

    pub fn bind_uniform<T>(&self, loc: ULoc, value: T) -> Result<(), JsValue>
    where
        T: WebGlType,
    {
        value.bind_uniform(&self.ctx, Some(&loc));

        return Ok(());
    }

    pub fn bind_array<T>(&self, loc: VLoc, array: &[T]) -> Result<(), JsValue>
    where
        T: WebGlType,
    {
        let (ctx, loc) = (&self.ctx, loc.0);

        let gl_buffer = ctx.create_buffer().ok_or("failed to create buffer")?;
        ctx.bind_buffer(BufferKind::Array as u32, Some(&gl_buffer));

        unsafe {
            let obj = T::view(array);

            // copies into buffer
            ctx.buffer_data_with_array_buffer_view(
                BufferKind::Array as u32,
                &obj,
                UsagePattern::Static as u32,
            );
        }

        ctx.enable_vertex_attrib_array(loc);

        let normal = false;
        let stride = 0; // if this is 0, we use the stride of the type
        let offset = 0;

        if T::is_int() {
            ctx.vertex_attrib_i_pointer_with_i32(loc, T::SIZE, T::GL_TYPE, stride, offset);
        } else {
            ctx.vertex_attrib_pointer_with_i32(loc, T::SIZE, T::GL_TYPE, normal, stride, offset);
        }

        return Ok(());
    }

    pub fn bind_tex(&self, loc: ULoc, unit: u32, rect: Rect, data: &[u8]) -> Result<(), JsValue> {
        let ctx = &self.ctx;

        let tex_type = Context::TEXTURE_2D;
        let data_type = Context::UNSIGNED_BYTE;
        let format = Context::LUMINANCE;

        ctx.pixel_storei(Context::UNPACK_ALIGNMENT, 1);

        ctx.uniform1i(Some(&loc), unit as i32);

        let tex = ctx.create_texture().unwrap();

        ctx.active_texture(Context::TEXTURE0 + unit);
        ctx.bind_texture(tex_type, Some(&tex));

        let filter_type = Context::NEAREST as i32;
        let wrap_type = Context::CLAMP_TO_EDGE as i32;

        ctx.tex_parameteri(tex_type, Context::TEXTURE_WRAP_S, wrap_type);
        ctx.tex_parameteri(tex_type, Context::TEXTURE_WRAP_T, wrap_type);
        ctx.tex_parameteri(tex_type, Context::TEXTURE_MIN_FILTER, filter_type);
        ctx.tex_parameteri(tex_type, Context::TEXTURE_MAG_FILTER, filter_type);

        ctx.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            tex_type,
            0,
            format as i32,
            rect.width as i32,
            rect.height as i32,
            0,
            format,
            data_type,
            Some(data),
        )?;

        return Ok(());
    }
}

fn compile_shader(
    context: &Context,
    shader_type: ShaderType,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type as u32)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    let success = context
        .get_shader_parameter(&shader, Context::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false);

    if success {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

fn link_program(
    context: &Context,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create program object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    let success = context
        .get_program_parameter(&program, Context::LINK_STATUS)
        .as_bool()
        .unwrap_or(false);

    if success {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

impl WebGlType for f32 {
    const GL_TYPE: u32 = Context::FLOAT;
    const SIZE: i32 = 1;

    unsafe fn view(array: &[Self]) -> Object {
        return js_sys::Float32Array::view(array).into();
    }

    fn bind_uniform(self, ctx: &Context, loc: Option<&web_sys::WebGlUniformLocation>) {
        ctx.uniform1f(loc, self);
    }
}

impl WebGlType for u32 {
    const GL_TYPE: u32 = Context::UNSIGNED_INT;
    const SIZE: i32 = 1;

    unsafe fn view(array: &[Self]) -> Object {
        return js_sys::Uint32Array::view(array).into();
    }

    fn bind_uniform(self, ctx: &Context, loc: Option<&web_sys::WebGlUniformLocation>) {
        ctx.uniform1ui(loc, self);
    }
}

#[wasm_bindgen(
    inline_js = "export function webglContext() { return { premultipliedAlpha: false }; }"
)]
extern "C" {
    #[wasm_bindgen(js_name = webglContext)]
    fn webgl_context() -> JsValue;
}
