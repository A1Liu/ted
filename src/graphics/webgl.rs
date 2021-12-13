use crate::util::*;
use js_sys::Object;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
pub use web_sys::WebGl2RenderingContext as Context;
pub use web_sys::WebGlBuffer;
pub use web_sys::WebGlTexture as Texture;
pub use web_sys::WebGlUniformLocation as ULoc;
pub use web_sys::WebGlVertexArrayObject as VAO;
use web_sys::{WebGlProgram, WebGlShader};

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

#[wasm_bindgen]
pub struct WebGl {
    phantom: (),
}

pub struct Program {
    program: WebGlProgram,
}

pub struct Buffer<T>
where
    T: WebGlType,
{
    buf: WebGlBuffer,
    phantom: core::marker::PhantomData<T>,
}

impl<T> Buffer<T>
where
    T: WebGlType,
{
    pub fn new(buf: WebGlBuffer) -> Self {
        return Self {
            buf,
            phantom: core::marker::PhantomData,
        };
    }
}

impl WebGl {
    pub fn use_program(&self, prog: &Program) {
        WEB_GL.with(|ctx| {
            ctx.use_program(Some(&prog.program));
        });
    }

    pub fn compile(&self, vert: &str, frag: &str) -> Result<Program, JsValue> {
        return WEB_GL.with(|ctx| {
            let vert_shader = compile_shader(ctx, ShaderType::Vertex, vert)?;
            let frag_shader = compile_shader(ctx, ShaderType::Fragment, frag)?;
            let program = link_program(ctx, &vert_shader, &frag_shader)?;

            return Ok(Program { program });
        });
    }

    pub fn vao(&self) -> Result<VAO, JsValue> {
        return WEB_GL.with(|ctx| {
            let vao = ctx.create_vertex_array().ok_or("Couldn't create VAO")?;
            ctx.bind_vertex_array(Some(&vao));

            return Ok(vao);
        });
    }

    pub fn bind_vao(&self, vao: &VAO) {
        return WEB_GL.with(|ctx| {
            ctx.bind_vertex_array(Some(vao));
        });
    }

    pub fn draw(&self, triangles: i32) {
        return WEB_GL.with(|ctx| {
            ctx.clear_color(0.0, 0.0, 0.0, 1.0);
            ctx.clear(Context::COLOR_BUFFER_BIT);
            ctx.draw_arrays(Context::TRIANGLES, 0, triangles);
        });
    }

    pub fn uloc(&self, prog: &Program, name: &str) -> Result<ULoc, JsValue> {
        return WEB_GL.with(|ctx| {
            let make_err = || format!("Failed to write uniform");
            let loc_opt = ctx.get_uniform_location(&prog.program, name);
            let loc = loc_opt.ok_or_else(make_err)?;

            return Ok(loc);
        });
    }

    pub fn bind_uniform<T>(&self, loc: &ULoc, value: T)
    where
        T: WebGlType,
    {
        WEB_GL.with(|ctx| {
            value.bind_uniform(ctx, Some(loc));
        });
    }

    pub fn write_buffer<T>(&self, buf: &Buffer<T>, data: &[T])
    where
        T: WebGlType,
    {
        WEB_GL.with(|ctx| {
            ctx.bind_buffer(BufferKind::Array as u32, Some(&buf.buf));

            unsafe {
                let obj = T::view(data);

                // copies into buffer
                ctx.buffer_data_with_array_buffer_view(
                    BufferKind::Array as u32,
                    &obj,
                    UsagePattern::Static as u32,
                );
            }
        });
    }

    pub fn attr_buffer<T>(&self, prog: &Program, name: &str) -> Result<Buffer<T>, JsValue>
    where
        T: WebGlType,
    {
        return WEB_GL.with(|ctx| {
            let loc = ctx.get_attrib_location(&prog.program, name);
            let make_err = |e| format!("failed to get location of '{}' (got {})", name, loc);
            let loc = loc.try_into().map_err(make_err)?;

            let gl_buffer = ctx.create_buffer().ok_or("failed to create buffer")?;
            ctx.bind_buffer(BufferKind::Array as u32, Some(&gl_buffer));
            ctx.enable_vertex_attrib_array(loc);

            if T::is_int() {
                ctx.vertex_attrib_i_pointer_with_i32(loc, T::SIZE, T::GL_TYPE, 0, 0);
            } else {
                ctx.vertex_attrib_pointer_with_i32(loc, T::SIZE, T::GL_TYPE, false, 0, 0);
            }

            return Ok(Buffer::new(gl_buffer));
        });
    }

    pub fn tex(&self, loc: &ULoc, unit: u32) -> Result<Texture, JsValue> {
        let tex_type = Context::TEXTURE_2D;
        let filter_type = Context::NEAREST as i32;
        let wrap_type = Context::CLAMP_TO_EDGE as i32;

        return WEB_GL.with(|ctx| {
            let tex = ctx.create_texture().ok_or("failed to create buffer")?;
            ctx.active_texture(Context::TEXTURE0);
            ctx.bind_texture(tex_type, Some(&tex));

            // These disable mip-mapping, which I do not want to deal with right now
            ctx.tex_parameteri(tex_type, Context::TEXTURE_WRAP_S, wrap_type);
            ctx.tex_parameteri(tex_type, Context::TEXTURE_WRAP_T, wrap_type);
            ctx.tex_parameteri(tex_type, Context::TEXTURE_MIN_FILTER, filter_type);
            ctx.tex_parameteri(tex_type, Context::TEXTURE_MAG_FILTER, filter_type);

            return Ok(tex);
        });
    }

    pub fn update_tex(&self, tex: &Texture, rect: Rect, data: &[u8]) -> Result<(), JsValue> {
        let tex_type = Context::TEXTURE_2D;
        let data_type = Context::UNSIGNED_BYTE;
        let format = Context::LUMINANCE;

        WEB_GL.with(|ctx| {
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
            )
        })?;

        return Ok(());
    }

    pub fn bind_tex(&self, loc: &ULoc, unit: u32, tex: &Texture) {
        let tex_type = Context::TEXTURE_2D;
        let data_type = Context::UNSIGNED_BYTE;
        let format = Context::LUMINANCE;

        return WEB_GL.with(|ctx| {
            ctx.uniform1i(Some(loc), unit as i32);
            ctx.active_texture(Context::TEXTURE0 + unit);
            ctx.bind_texture(tex_type, Some(tex));
        });
    }
}

#[repr(u32)]
enum ShaderType {
    Vertex = Context::VERTEX_SHADER,
    Fragment = Context::FRAGMENT_SHADER,
}

#[repr(u32)]
enum BufferKind {
    Array = Context::ARRAY_BUFFER,
}

#[repr(u32)]
enum UsagePattern {
    Static = Context::STATIC_DRAW,
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
    ctx: &Context,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = ctx
        .create_program()
        .ok_or_else(|| String::from("Unable to create program object"))?;

    ctx.attach_shader(&program, vert_shader);
    ctx.attach_shader(&program, frag_shader);
    ctx.link_program(&program);

    let success = ctx
        .get_program_parameter(&program, Context::LINK_STATUS)
        .as_bool()
        .unwrap_or(false);

    if success {
        Ok(program)
    } else {
        Err(ctx
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

pub static gl: WebGl = WebGl { phantom: () };

thread_local! {
    static OFFSCREEN_CANVAS: web_sys::HtmlCanvasElement = get_canvas().unwrap();
    pub static WEB_GL: Context = webgl_ctx().unwrap();
}

fn get_canvas() -> Result<web_sys::HtmlCanvasElement, JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    return Ok(canvas);
}

fn webgl_ctx() -> Result<Context, JsValue> {
    return OFFSCREEN_CANVAS.with(|canvas| {
        let options = webgl_context_options();

        let ctx = canvas
            .get_context_with_context_options("webgl2", &options)?
            .unwrap()
            .dyn_into::<Context>()?;

        ctx.pixel_storei(Context::UNPACK_ALIGNMENT, 1);

        return Ok(ctx);
    });
}

#[wasm_bindgen(
    inline_js = "export function webglContext() { return { premultipliedAlpha: false }; }"
)]
extern "C" {
    #[wasm_bindgen(js_name = webglContext)]
    fn webgl_context_options() -> JsValue;
}
