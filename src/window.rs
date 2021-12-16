use crate::graphics::*;
use crate::util::*;
use winit::event;
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopProxy};
use winit::platform::web::WindowBuilderExtWebSys;
use winit::window::{Window, WindowBuilder, WindowId};

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum TedEvent {
    Tick(usize),
}

const TEXT: &'static str = r#"Welcome to my stupid project to make a text editor.
Try typing!
"#;

// TODO pass in the canvas we wanna use
pub fn start_window() -> ! {
    // Because of how the event loop works, values in this outer scope do not
    // get dropped. They either get captured or forgotten.

    let event_loop: EventLoop<TedEvent> = EventLoop::with_user_event();

    {
        let event_loop_proxy = event_loop.create_proxy();
        setup_tick(event_loop_proxy);
    }

    let mut handler = {
        let canvas = expect(get_canvas());
        let window = WindowBuilder::new()
            .with_canvas(Some(canvas))
            .build(&event_loop)
            .unwrap();

        Handler::new(window)
    };

    handler.redraw(handler.window.id());

    event_loop.run(move |event, _, flow| {
        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        *flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, window_id } => handler.window_event(flow, event, window_id),
            Event::UserEvent(ted_event) => handler.ted_event(ted_event),
            Event::RedrawRequested(window_id) => handler.redraw(window_id),
            _ => (),
        }
    });
}

struct Handler {
    window: Window,
    window_dims: Rect,
    text: String,
    cache: GlyphCache,
    cursor_pos: Point2<u32>,
    cursor_on: bool,
}

impl Handler {
    fn new(window: Window) -> Self {
        return Self {
            window,
            window_dims: Rect::new(28, 15),
            text: String::from(TEXT),
            cache: GlyphCache::new(),
            cursor_pos: Point2 { x: 0, y: 0 },
            cursor_on: true,
        };
    }

    fn redraw(&mut self, window_id: WindowId) {
        let cursor_pos = match self.cursor_on {
            true => Some(self.cursor_pos),
            false => None,
        };
        let mut vertices = TextVertices::new(&mut self.cache, self.window_dims, cursor_pos);
        vertices.push(&self.text);
        expect(vertices.render());
    }

    fn ted_event(&mut self, evt: TedEvent) {
        match evt {
            TedEvent::Tick(tick) => {
                if tick % 12 == 0 {
                    self.cursor_on = !self.cursor_on;
                    self.window.request_redraw();
                }
            }
        }
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

                // https://docs.rs/winit/0.26.0/winit/event/enum.VirtualKeyCode.html
                let key = match input.virtual_keycode {
                    Some(key) => key,
                    None => return,
                };

                // We need to use this right now because the alternative literally isn't
                // implemented on the web
                #[allow(deprecated)]
                let modifiers = input.modifiers;

                if modifiers.ctrl() || modifiers.logo() || modifiers.alt() {
                    return;
                }

                self.cursor_on = true;
                self.window.request_redraw();

                match key {
                    event::VirtualKeyCode::Left => {
                        self.cursor_pos.x = self.cursor_pos.x.saturating_sub(1);
                        return;
                    }
                    event::VirtualKeyCode::Up => {
                        self.cursor_pos.y = self.cursor_pos.y.saturating_sub(1);
                        return;
                    }
                    event::VirtualKeyCode::Right => {
                        if self.cursor_pos.x < self.window_dims.width - 1 {
                            self.cursor_pos.x += 1;
                        }
                        return;
                    }
                    event::VirtualKeyCode::Down => {
                        if self.cursor_pos.y < self.window_dims.height - 1 {
                            self.cursor_pos.y += 1;
                        }
                        return;
                    }
                    _ => {}
                }

                let c = match keycode_char(modifiers, key) {
                    Some(c) => c,
                    None => return,
                };

                self.text.push(c);

                self.window.request_redraw();
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

#[cfg(target_arch = "wasm32")]
pub fn setup_tick(proxy: EventLoopProxy<TedEvent>) {
    let window = unwrap(web_sys::window());
    let mut ticks = 0;

    let closure = enclose(move || {
        expect(proxy.send_event(TedEvent::Tick(ticks)));
        ticks += 1;

        return Ok(());
    });

    repeat(&closure, 16);

    closure.forget();
}

#[wasm_bindgen(inline_js = r#"
export const repeat = async (func, ms) => {
  while (true) {
    func();
    await new Promise((res) => setTimeout(res, ms));
  }
};
"#)]
extern "C" {
    fn repeat(func: &Closure<JsFunc>, ms: i32);
}
