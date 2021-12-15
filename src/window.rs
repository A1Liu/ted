use crate::graphics::*;
use crate::util::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use winit::event;
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::web::WindowBuilderExtWebSys;
use winit::window::{Window, WindowBuilder, WindowId};

enum TedEvent {
    TimerTick(usize),
}

const TEXT: &'static str = r#"Welcome to my stupid project to make a text editor.
And now, Kirin J. Callinan's "Big Enough":
"#;

// TODO pass in the canvas we wanna use
pub fn start_window() -> ! {
    // Because of how the event loop works, values in this outer scope do not
    // get dropped. They either get captured or forgotten.

    let event_loop = EventLoop::new();

    let mut handler = {
        let canvas = expect(get_canvas());
        let window = WindowBuilder::new()
            .with_canvas(Some(canvas))
            .build(&event_loop)
            .unwrap();

        Handler::new(window)
    };

    {
        let mut vertices = TextVertices::new(&mut handler.cache, 28, 15);
        vertices.push(&handler.text);
        expect(vertices.render());
    }

    event_loop.run(move |event, _, flow| {
        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        *flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, window_id } => handler.window_event(flow, event, window_id),
            _ => (),
        }
    });
}

struct Handler {
    window: Window,
    text: String,
    cache: GlyphCache,
}

impl Handler {
    fn new(window: Window) -> Self {
        return Self {
            window,
            text: String::from(TEXT),
            cache: GlyphCache::new(),
        };
    }

    fn window_event(&mut self, flow: &mut ControlFlow, event: WindowEvent, id: WindowId) {
        match event {
            WindowEvent::CloseRequested => {
                if self.window.id() == id {
                    *flow = ControlFlow::Exit;
                }
            }

            WindowEvent::KeyboardInput {
                device_id,
                input,
                is_synthetic,
            } => {
                if input.state != ElementState::Pressed {
                    return;
                }

                let key = match input.virtual_keycode {
                    Some(key) => key,
                    None => return,
                };

                // We need to use this right now because the alternative literally isn't
                // implemented on the web
                #[allow(deprecated)]
                let modifiers = input.modifiers;

                if modifiers.ctrl() || modifiers.ctrl() || modifiers.alt() {
                    return;
                }

                let c = match keycode_char(modifiers, key) {
                    Some(c) => c,
                    None => return,
                };

                self.text.push(c);

                let mut vertices = TextVertices::new(&mut self.cache, 28, 15);
                vertices.push(&self.text);
                expect(vertices.render());
            }

            _ => {}
        }
    }
}

pub fn get_canvas() -> Result<web_sys::HtmlCanvasElement, JsValue> {
    let window = unwrap(web_sys::window());
    let document = unwrap(window.document());
    let canvas = unwrap(document.get_element_by_id("canvas"));
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    return Ok(canvas);
}

// https://docs.rs/winit/0.26.0/winit/event/enum.VirtualKeyCode.html
fn keycode_char(modifiers: event::ModifiersState, key: event::VirtualKeyCode) -> Option<char> {
    use event::VirtualKeyCode::*;

    let shift = modifiers.shift();

    macro_rules! shift {
        ($lower:expr, $upper:expr) => {{
            if shift {
                $upper
            } else {
                $lower
            }
        }};
    }

    let c = match key {
        A => shift!('a', 'A'),
        B => shift!('b', 'B'),
        C => shift!('c', 'C'),
        D => shift!('d', 'D'),
        E => shift!('e', 'E'),
        F => shift!('f', 'F'),
        G => shift!('g', 'G'),
        H => shift!('h', 'H'),
        I => shift!('i', 'I'),
        J => shift!('j', 'J'),
        K => shift!('k', 'K'),
        L => shift!('l', 'L'),
        M => shift!('m', 'M'),
        N => shift!('n', 'N'),
        O => shift!('o', 'O'),
        P => shift!('p', 'P'),
        Q => shift!('q', 'Q'),
        R => shift!('r', 'R'),
        S => shift!('s', 'S'),
        T => shift!('t', 'T'),
        U => shift!('u', 'U'),
        V => shift!('v', 'V'),
        W => shift!('w', 'W'),
        X => shift!('x', 'X'),
        Y => shift!('y', 'Y'),
        Z => shift!('z', 'Z'),

        Key1 => shift!('1', '!'),
        Key2 => shift!('2', '@'),
        Key3 => shift!('3', '#'),
        Key4 => shift!('4', '$'),
        Key5 => shift!('5', '%'),
        Key6 => shift!('6', '^'),
        Key7 => shift!('7', '&'),
        Key8 => shift!('8', '*'),
        Key9 => shift!('9', '('),
        Key0 => shift!('0', ')'),

        Return => '\n',
        Space => ' ',

        _ => return None,
    };

    return Some(c);
}
