use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::web::WindowBuilderExtWebSys;
use winit::window::{Window, WindowBuilder};

// TODO pass in the canvas we wanna use
pub fn start_window() -> ! {
    let canvas = get_canvas().unwrap();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_canvas(Some(canvas))
        .build(&event_loop)
        .unwrap();

    event_loop.run(move |event, _, flow| {
        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        *flow = ControlFlow::Wait;

        let id = window.id();

        match event {
            Event::WindowEvent { event, window_id } => {
                if event == WindowEvent::CloseRequested && window_id == id {
                    *flow = ControlFlow::Exit;
                }
            }
            _ => (),
        }
    });
}

pub fn get_canvas() -> Result<web_sys::HtmlCanvasElement, JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    return Ok(canvas);
}
