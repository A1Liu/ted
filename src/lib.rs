// Long-term
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_macros)]
#![allow(unused_braces)]
#![allow(non_upper_case_globals)]
// Short-term allows
/* */
#![allow(unused_imports)]
#![allow(unused_mut)]
/* */

#[cfg(target_arch = "wasm32")]
#[macro_use]
mod print_utils;

mod text;
mod util;

#[cfg(target_arch = "wasm32")]
mod view;

#[cfg(target_arch = "wasm32")]
mod event_loop;

#[cfg(target_arch = "wasm32")]
mod graphics;

#[cfg(target_arch = "wasm32")]
pub use wasm_exports::*;

#[cfg(target_arch = "wasm32")]
mod wasm_exports {
    use crate::event_loop::*;
    use crate::util::*;
    use winit::event_loop::{EventLoop, EventLoopProxy};
    use winit::platform::web::WindowBuilderExtWebSys;
    use winit::window::WindowBuilder;

    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

    const TEXT: &'static str = r#"Welcome to my stupid project to make a text editor.
Try typing!
"#;

    #[wasm_bindgen]
    pub fn start() {
        #[cfg(debug_assertions)]
        {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        }

        let f = enclose(move || {
            let event_loop: EventLoop<TedEvent> = EventLoop::with_user_event();

            // Because of how the event loop works, values in this outer scope do not
            // get dropped. They either get captured or forgotten.

            {
                let event_loop_proxy = event_loop.create_proxy();
                setup_tick(event_loop_proxy);
            }

            let handler = {
                let canvas = expect(get_canvas());
                let window = WindowBuilder::new()
                    .with_canvas(Some(canvas))
                    .build(&event_loop)
                    .unwrap();

                Handler::new(window, TEXT.to_string())
            };

            event_loop.run(handler.into_runner());
        });
        prevent_throw(&f);
    }

    fn setup_tick(proxy: EventLoopProxy<TedEvent>) {
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
export const preventThrow = (fn) => {
  try {
    fn();
  } catch (e) {}
};

export const repeat = async (func, ms) => {
  while (true) {
    func();
    await new Promise((res) => setTimeout(res, ms));
  }
};
"#)]
    extern "C" {
        #[wasm_bindgen(js_name = "preventThrow")]
        fn prevent_throw(func: &Closure<dyn FnMut() -> Result<(), JsValue>>);

        fn repeat(func: &Closure<JsFunc>, ms: i32);
    }
}
