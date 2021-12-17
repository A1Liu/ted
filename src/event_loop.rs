use crate::graphics::*;
use crate::util::*;
use winit::event;
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowBuilder, WindowId};

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum TedEvent {
    Tick(usize),
}

pub struct Handler {
    window: Window,
    window_dims: Rect,
    text: String,
    cache: GlyphCache,
    cursor_pos: Point2<u32>,
    cursor_on: bool,
}

impl Handler {
    pub fn new(window: Window, text: String) -> Self {
        return Self {
            window,
            window_dims: new_rect(28, 15),
            text,
            cache: GlyphCache::new(),
            cursor_pos: Point2 { x: 0, y: 0 },
            cursor_on: true,
        };
    }

    pub fn into_runner(
        mut self,
    ) -> impl 'static
           + FnMut(
        Event<'_, TedEvent>,
        &winit::event_loop::EventLoopWindowTarget<TedEvent>,
        &mut ControlFlow,
    ) {
        return move |event, _, flow| {
            // ControlFlow::Wait pauses the event loop if no events are available to process.
            // This is ideal for non-game applications that only update in response to user
            // input, and uses significantly less power/CPU time than ControlFlow::Poll.
            *flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent { event, window_id } => {
                    self.window_event(flow, event, window_id)
                }
                Event::UserEvent(ted_event) => self.ted_event(ted_event),
                Event::RedrawRequested(window_id) => self.redraw(window_id),
                _ => (),
            }
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
                        if self.cursor_pos.x < self.window_dims.x - 1 {
                            self.cursor_pos.x += 1;
                        }
                        return;
                    }
                    event::VirtualKeyCode::Down => {
                        if self.cursor_pos.y < self.window_dims.y - 1 {
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
