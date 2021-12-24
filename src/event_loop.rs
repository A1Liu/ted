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

            let mut commands: Vec<TedCommand> = Vec::new();

            match event {
                Event::WindowEvent { event, window_id } => {
                    self.window_event(event, window_id, &mut commands)
                }
                Event::UserEvent(ted_event) => self.ted_event(ted_event, &mut commands),
                Event::RedrawRequested(window_id) => self.draw(window_id, &mut commands),
                _ => (),
            }

            self.command_handler.run(&self.window, flow, commands);
        };
    }

    fn draw(&mut self, window_id: WindowId, commands: &mut Vec<TedCommand>) {
        commands.push(TedCommand::Draw);
    }

    fn ted_event(&mut self, evt: TedEvent, commands: &mut Vec<TedCommand>) {
        match evt {
            TedEvent::Tick(tick) => {
                if tick % 12 == 0 {
                    commands.push(TedCommand::ForView {
                        command: ViewCommand::ToggleCursorBlink,
                    });
                    // self.view.toggle_cursor_blink(commands);
                }
            }
        }
    }

    fn window_event(&mut self, event: WindowEvent, id: WindowId, commands: &mut Vec<TedCommand>) {
        match event {
            WindowEvent::CloseRequested => {
                if self.window.id() == id {
                    commands.push(TedCommand::Exit);
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

                if let Some(direction) = Direction::from_arrow_key(key) {
                    commands.push(TedCommand::ForView {
                        command: ViewCommand::CursorMove(direction),
                    });
                    return;
                }

                if key == event::VirtualKeyCode::Back {
                    commands.push(TedCommand::ForView {
                        command: ViewCommand::Delete,
                    });
                    return;
                }

                let c = match keycode_str(modifiers, key) {
                    Some(c) => c,
                    None => return,
                };

                commands.push(TedCommand::ForView {
                    command: ViewCommand::Insert { text: c },
                });
            }

            _ => {}
        }
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
