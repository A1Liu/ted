use js_sys::Object;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGlRenderingContext as Context, WebGlShader};

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

    fn bind_uniform(self, ctx: &Context, loc: Option<&web_sys::WebGlUniformLocation>) {
        unimplemented!();
    }
}

// Maybe later make this more convenient to use with multiple programs
pub struct WebGl {
    ctx: Context,
    program: WebGlProgram,
}

impl WebGl {
    pub fn new(canvas: web_sys::Element) -> Result<Self, JsValue> {
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
        let ctx = canvas
            .get_context("webgl")?
            .unwrap()
            .dyn_into::<Context>()?;

        let vert_text = core::include_str!("./vertex.glsl");
        let vert_shader = compile_shader(&ctx, ShaderType::Vertex, vert_text)?;

        let frag_text = core::include_str!("./fragment.glsl");
        let frag_shader = compile_shader(&ctx, ShaderType::Fragment, frag_text)?;

        let program = link_program(&ctx, &vert_shader, &frag_shader)?;
        ctx.use_program(Some(&program));

        return Ok(WebGl { ctx, program });
    }

    pub fn draw(&self, triangles: i32) {
        self.ctx.clear_color(0.0, 0.0, 0.0, 1.0);
        self.ctx.clear(Context::COLOR_BUFFER_BIT);
        self.ctx.draw_arrays(Context::TRIANGLES, 0, triangles);
    }

    pub fn bind_uniform<T>(&self, name: &'static str, value: T) -> Result<(), JsValue>
    where
        T: WebGlType,
    {
        let make_err = || JsValue::from("Failed to write uniform");
        let loc_opt = self.ctx.get_uniform_location(&self.program, name);
        let loc = loc_opt.ok_or_else(make_err)?;
        value.bind_uniform(&self.ctx, Some(&loc));

        return Ok(());
    }

    pub fn bind_array<T>(&self, attrib: &'static str, array: &[T]) -> Result<(), JsValue>
    where
        T: WebGlType,
    {
        let (ctx, program) = (&self.ctx, &self.program);

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
