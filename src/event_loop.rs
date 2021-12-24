use crate::data::*;
use crate::graphics::*;
use crate::text::*;
use crate::util::*;
use crate::view::*;
use winit::event;
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::window::{Window, WindowId};

pub struct Handler {
    // This should be global
    cache: GlyphCache,

    // These eventually should not be
    window: Window,
    view: View,
    file: File,
}

impl Handler {
    pub fn new(window: Window, text: String) -> Self {
        let mut cache = GlyphCache::new();
        let view = View::new(new_rect(28, 15), &mut cache);
        let mut file = File::new();
        file.push_str(&text);

        return Self {
            cache,
            window,
            view,
            file,
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
                Event::RedrawRequested(window_id) => self.draw(window_id),
                _ => (),
            }
        };
    }

    fn draw(&mut self, window_id: WindowId) {
        self.view.draw(&mut self.file, &mut self.cache);
    }

    fn ted_event(&mut self, evt: TedEvent) {
        match evt {
            TedEvent::Tick(tick) => {
                if tick % 12 == 0 {
                    self.view.toggle_cursor_blink(&self.window);
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

                if self.view.cursor_move(&self.window, key) {
                    return;
                }

                if key == event::VirtualKeyCode::Back {
                    self.view.delete(&self.window, &mut self.file);
                    return;
                }

                let c = match keycode_char(modifiers, key) {
                    Some(c) => c,
                    None => return,
                };

                self.view.insert_char(&self.window, &mut self.file, c);
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
