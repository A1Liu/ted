use crate::command_handler::*;
use crate::commands::*;
use crate::graphics::*;
use crate::text::*;
use crate::util::*;
use crate::view::*;
use winit::event;
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::window::{Window, WindowId};

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum TedEvent {
    Tick(usize),
}

pub struct Handler {
    command_handler: CommandHandler,
    window: Window,
}

impl Handler {
    pub fn new(window: Window, text: String) -> Self {
        return Self {
            command_handler: CommandHandler::new(text),
            window,
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

            let command = match self.dispatch(event) {
                None => return,
                Some(c) => c,
            };

            self.command_handler.run(&self.window, flow, command);
        };
    }

    fn dispatch(&mut self, event: Event<TedEvent>) -> Option<TedCommand> {
        let command = match event {
            Event::WindowEvent { event, window_id } => self.window_event(event, window_id)?,
            Event::UserEvent(ted_event) => self.ted_event(ted_event)?,
            Event::RedrawRequested(window_id) => for_view(ViewCommand::Draw),
            _ => return None,
        };

        return Some(command);
    }

    fn ted_event(&mut self, evt: TedEvent) -> Option<TedCommand> {
        match evt {
            TedEvent::Tick(tick) => {
                if tick % 12 == 0 {
                    return Some(for_view(ViewCommand::ToggleCursorBlink));
                }
            }
        }

        return None;
    }

    fn window_event(&mut self, event: WindowEvent, id: WindowId) -> Option<TedCommand> {
        match event {
            WindowEvent::CloseRequested => {
                if self.window.id() == id {
                    return Some(TedCommand::Exit);
                }
            }

            WindowEvent::KeyboardInput {
                device_id,
                input,
                is_synthetic,
            } => {
                if input.state != ElementState::Pressed {
                    return None;
                }

                // https://docs.rs/winit/0.26.0/winit/event/enum.VirtualKeyCode.html
                let key = input.virtual_keycode?;

                // We need to use this right now because the alternative literally isn't
                // implemented on the web
                #[allow(deprecated)]
                let modifiers = input.modifiers;

                if modifiers.ctrl() || modifiers.logo() || modifiers.alt() {
                    return None;
                }

                if let Some(direction) = Direction::from_arrow_key(key) {
                    return Some(for_view(ViewCommand::CursorMove(direction)));
                }

                if key == event::VirtualKeyCode::Back {
                    return Some(for_view(ViewCommand::DeleteAfterCursor));
                }

                let text = keycode_str(modifiers, key)?.to_string();

                return Some(for_view(ViewCommand::Insert { text }));
            }

            _ => {}
        }

        return None;
    }
}

// https://docs.rs/winit/0.26.0/winit/event/enum.VirtualKeyCode.html
fn keycode_str(
    modifiers: event::ModifiersState,
    key: event::VirtualKeyCode,
) -> Option<&'static str> {
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
        A => shift!("a", "A"),
        B => shift!("b", "B"),
        C => shift!("c", "C"),
        D => shift!("d", "D"),
        E => shift!("e", "E"),
        F => shift!("f", "F"),
        G => shift!("g", "G"),
        H => shift!("h", "H"),
        I => shift!("i", "I"),
        J => shift!("j", "J"),
        K => shift!("k", "K"),
        L => shift!("l", "L"),
        M => shift!("m", "M"),
        N => shift!("n", "N"),
        O => shift!("o", "O"),
        P => shift!("p", "P"),
        Q => shift!("q", "Q"),
        R => shift!("r", "R"),
        S => shift!("s", "S"),
        T => shift!("t", "T"),
        U => shift!("u", "U"),
        V => shift!("v", "V"),
        W => shift!("w", "W"),
        X => shift!("x", "X"),
        Y => shift!("y", "Y"),
        Z => shift!("z", "Z"),

        Key1 => shift!("1", "!"),
        Key2 => shift!("2", "@"),
        Key3 => shift!("3", "#"),
        Key4 => shift!("4", "$"),
        Key5 => shift!("5", "%"),
        Key6 => shift!("6", "^"),
        Key7 => shift!("7", "&"),
        Key8 => shift!("8", "*"),
        Key9 => shift!("9", "("),
        Key0 => shift!("0", ")"),

        Return => "\n",
        Space => " ",

        _ => return None,
    };

    return Some(c);
}
