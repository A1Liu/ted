#![allow(dead_code)]
#![allow(unused_variables)]

pub mod fonts;
pub mod graphics;
pub mod large_text;
pub mod text;
pub mod util;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};

#[rustfmt::skip] // rustfmt deletes the keyword async
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(a: &str);
}

#[wasm_bindgen]
pub fn albert_editor_main_loop() {
    log("Hello World!\n");
}

#[wasm_bindgen]
pub fn render(canvas: web_sys::Element) -> Result<(), JsValue> {
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let context = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()?;

    let vert_shader = compile_shader(
        &context,
        WebGlRenderingContext::VERTEX_SHADER,
        core::include_str!("./vertex.glsl"),
    )?;
    let frag_shader = compile_shader(
        &context,
        WebGlRenderingContext::FRAGMENT_SHADER,
        core::include_str!("./fragment.glsl"),
    )?;
    let program = link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));

    let vertices: [f32; 9] = [-0.7, -0.7, 0.0, 0.7, -0.7, 0.0, 0.0, 0.7, 0.0];
    vertex_buffer(&context, &program, &vertices, "position")?;

    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    context.draw_arrays(
        WebGlRenderingContext::TRIANGLES,
        0,
        (vertices.len() / 3) as i32,
    );

    Ok(())
}

fn texture_buffer(
    ctx: &WebGlRenderingContext,
    program: &WebGlProgram,
    buffer: &[u8],
    attrib: &'static str,
) -> Result<(), JsValue> {
    return Ok(());
}

fn vertex_buffer(
    ctx: &WebGlRenderingContext,
    program: &WebGlProgram,
    buffer: &[f32],
    attrib: &'static str,
) -> Result<(), JsValue> {
    let gl_buffer = ctx.create_buffer().ok_or("failed to create buffer")?;
    ctx.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&gl_buffer));

    unsafe {
        let vert_array = js_sys::Float32Array::view(buffer);

        // copies into buffer
        ctx.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }

    let loc = ctx.get_attrib_location(program, attrib);
    if loc < 0 {
        return Err(JsValue::from("Failed to get location of variable"));
    }

    let loc = loc as u32;
    let size = 3;
    let attr_type = WebGlRenderingContext::FLOAT;
    let normal = false;
    let stride = 0; // if this is 0, we use the stride of the type
    let offset = 0;

    ctx.vertex_attrib_pointer_with_i32(loc, size, attr_type, normal, stride, offset);
    ctx.enable_vertex_attrib_array(loc);

    return Ok(());
}

fn compile_shader(
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
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

fn link_program(
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
