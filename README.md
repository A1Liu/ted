# Ted
Text editor using WebGL and Rust/Wasm. Very much WIP.

## Feature Stuffs
- [ ] https://github.com/cmuratori/refterm
- [ ] https://docs.rs/winit/latest/winit/
- [x] https://rustwasm.github.io/wasm-bindgen/examples/request-animation-frame.html
- [x] https://docs.rs/ttf-parser/latest/ttf_parser/
- [x] https://github.com/raphlinus/font-rs
- [x] https://webgl2fundamentals.org/webgl/lessons/webgl-how-it-works.html
- [ ] Client-server https://github.com/knsd/daemonize

## Size Opt Stuffs
- [ ] https://github.com/rustwasm/wasm-pack/issues/737
- [ ] https://github.com/johnthagen/min-sized-rust
- [ ] https://www.skypack.dev/view/esbuild-plugin-wasm-pack

## Graphics Opt Stuffs
- [ ] https://webgl2fundamentals.org/webgl/lessons/webgl-drawing-without-data.html
- [ ] https://developer.mozilla.org/en-US/docs/Web/API/WebGL2RenderingContext/bindBufferRange
- [ ] https://medium.com/@evanwallace/easy-scalable-text-rendering-on-the-gpu-c3f4d782c5ac
- [ ] https://webgl2fundamentals.org/webgl/lessons/webgl-qna-how-to-use-the-stencil-buffer.html
- [ ] https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices

## Next Up
- Deleting text, BTree Garbage collection
- Make text less ugly
- Line numbers
- Select text
- Window sizing and whatnot

## Mid-term
- Test system using commands and data-in/data-out
- Find, Find and replace
- Multiple canvases on the web
- Text colors/spans
- Graphics/shader stuff
- Persist data and whatnot

## Far in the Future
- Customization?
- Syntax highlighting; probably just make something super duper simple
- Client-server architecture so that we can have nice things
- Cross platform stuffs

## Even Further in the Future
- Python-style indexable UTF-8 strings (only implement single-byte variant for now)
- Abstract away graphics stuff with cute macros and stuff
- Cursor ref-counting or whatever works to get the behavior that Google docs has


## Architecture
- Windows for rendering, tightly connected to graphics for each system
  - Windows have views into files that store stuff like where you cursor is
  - Views do word wrapping by creating a string and inserting newlines, windows
    render as if there is no word wrapping, and cut off stuff when necessary
- Files are managed globally, you call functions to modify the files and those
  functions might end up being IPC calls or whatever, to support multiple windows
- Editor uses a command system for editor mutations. Like, literally, just use
  an enum to dispatch events. That way its easy to swap out an implementation
  with another
- Window doesn't do anything by default. It only dispatches events, and when certain
  events come in, it runs those events
