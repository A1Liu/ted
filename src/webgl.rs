use js_sys::Object;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};

#[repr(u32)]
pub enum ShaderType {
    Vertex = WebGlRenderingContext::VERTEX_SHADER,
    Fragment = WebGlRenderingContext::FRAGMENT_SHADER,
}

#[repr(u32)]
pub enum BufferKind {
    Array = WebGlRenderingContext::ARRAY_BUFFER,
}

#[repr(u32)]
pub enum DrawKind {
    Static = WebGlRenderingContext::STATIC_DRAW,
}

pub trait WebGlType
where
    Self: Sized,
{
    const GL_TYPE: u32;
    const SIZE: i32;

    unsafe fn view(array: &[Self]) -> Object;
}

pub struct WebGl {
    ctx: WebGlRenderingContext,
    program: WebGlProgram,
}

impl WebGl {
    pub fn new(canvas: web_sys::Element) -> Result<Self, JsValue> {
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
        let ctx = canvas
            .get_context("webgl")?
            .unwrap()
            .dyn_into::<WebGlRenderingContext>()?;

        let vert_text = core::include_str!("./vertex.glsl");
        let vert_shader = compile_shader(&ctx, ShaderType::Vertex, vert_text)?;

        let frag_text = core::include_str!("./fragment.glsl");
        let frag_shader = compile_shader(&ctx, ShaderType::Fragment, frag_text)?;

        let program = link_program(&ctx, &vert_shader, &frag_shader)?;
        ctx.use_program(Some(&program));

        return Ok(WebGl { ctx, program });
    }

    pub fn bind_array<T>(&self, attrib: &'static str, array: &[T]) -> Result<(), JsValue>
    where
        T: WebGlType,
    {
        let (ctx, program) = (&self.ctx, &self.program);

        new_buffer(ctx)?;

        unsafe {
            let obj = T::view(array);

            // copies into buffer
            ctx.buffer_data_with_array_buffer_view(
                BufferKind::Array as u32,
                &obj,
                DrawKind::Static as u32,
            );
        }

        let loc = ctx.get_attrib_location(program, attrib);
        if loc < 0 {
            return Err(JsValue::from("Failed to get location of variable"));
        }

        let loc = loc as u32;
        let normal = false;
        let stride = 0; // if this is 0, we use the stride of the type
        let offset = 0;

        ctx.vertex_attrib_pointer_with_i32(loc, T::SIZE, T::GL_TYPE, normal, stride, offset);
        ctx.enable_vertex_attrib_array(loc);

        return Ok(());
    }
}

pub fn new_buffer(ctx: &WebGlRenderingContext) -> Result<(), JsValue> {
    let gl_buffer = ctx.create_buffer().ok_or("failed to create buffer")?;
    ctx.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&gl_buffer));
    return Ok(());
}

pub fn compile_shader(
    context: &WebGlRenderingContext,
    shader_type: ShaderType,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type as u32)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    let success = context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
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

pub fn link_program(
    context: &WebGlRenderingContext,
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
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
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
