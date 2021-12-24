# Ted, the editor
Text editor using WebGL and Rust/Wasm. Very much WIP.

## Feature Stuffs
- [ ] https://github.com/cmuratori/refterm
- [ ] https://docs.rs/winit/latest/winit/
- [x] https://rustwasm.github.io/wasm-bindgen/examples/request-animation-frame.html
- [x] https://docs.rs/ttf-parser/latest/ttf_parser/
- [x] https://github.com/raphlinus/font-rs
- [x] https://webgl2fundamentals.org/webgl/lessons/webgl-how-it-works.html
- [ ] Client-server https://github.com/knsd/daemonize
- [ ] Sending data and whatnot https://github.com/capnproto/capnproto-rust
  - Ideally we can just send bytes over the wire in whatever format, validate it
    all at once on receipt, then convert it in-place into Rust compatible repr(C)
    structs. Would be easy to use, and would not require nonsense code bloat.
    In the interim though, we can do a runtime conversion to get similar productivity
    benefits.
  - Not using Serde because of resulting binary size and increased compile times.

## Size Opt Stuffs
- [ ] https://github.com/rustwasm/wasm-pack/issues/737
- [ ] https://github.com/johnthagen/min-sized-rust
- [ ] https://www.skypack.dev/view/esbuild-plugin-wasm-pack

## Graphics Opt Stuffs
- [x] https://webgl2fundamentals.org/webgl/lessons/webgl-drawing-without-data.html
- [ ] https://developer.mozilla.org/en-US/docs/Web/API/WebGL2RenderingContext/bindBufferRange
- [ ] https://medium.com/@evanwallace/easy-scalable-text-rendering-on-the-gpu-c3f4d782c5ac
- [ ] https://webgl2fundamentals.org/webgl/lessons/webgl-qna-how-to-use-the-stencil-buffer.html
- [ ] https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices

## Next Up
- Line numbers
- Test command system
- Text colors/spans
- Mouse support
- Make text less ugly
- Select text

## Mid-term
- BTree Garbage collection
- Find and replace
- Multiple canvases on the web
- Graphics/shader stuff
- Window sizing and whatnot
- Persist data and whatnot

## Far in the Future
- Customization?
- Syntax highlighting; probably just make something super duper simple
- Client-server architecture so that we can have nice things
- Cross platform stuffs

## Even Further in the Future
- Full unicode support?
- Python-style indexable UTF-8 strings
- Abstract away graphics stuff with cute macros and stuff
- Cursor ref-counting or whatever works to get the behavior that Google docs has

## Architecture
- Windows for rendering, tightly connected to graphics for each system
  - Windows have views into files that store stuff like where you cursor is
  - Window doesn't do anything by default. It only dispatches events, and when certain
    events come in, it runs those events
  - Views do word wrapping and create data for the window to render
- Files are managed globally, you call functions to modify the files and those
  functions might end up being IPC calls or whatever, to support multiple windows
- Editor uses a command system for editor mutations. Like, literally, just use
  an enum to dispatch mutations. That way its easy to swap out an implementation
  with another, and also testing becomes a data-in/data-out problem instead of
  a mocking/contracts/whatever problem
