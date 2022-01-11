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
  - Going to try to avoid Serde because of resulting binary size and increased
    compile times. Current binary size is 235KB, which is pretty gigantic for
    how little work it does, and a TodoMVC app with Serde is 476Kb. Runtime serialization
    performance is low priority for the message sizes this project works with,
    but avoiding a gigantic download size is an incredibly high priority.

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
- Mouse support
- Test the command system
- Simplify font rendering stuffs
- Select text
- Serialization using less binary bloat? Maybe just DIY it. There is no need to
  use the visitor pattern or support serializing to any arbitrary thing.

## Mid-term
- Refactor text flowing to have enough flexibility for the line wrapping issue
  and ideally more stuff in the future
- BTree Garbage collection
- Make graphics cross platform maybe?
- More advanced syntax highlighting; scopes, more context info, more generic kinds of rules
- Find and replace
- Switch to straight-line code as much as physically possible
- Multiple canvases on the web
- More Graphics/shader stuff
- Window sizing and whatnot
- Persist data and whatnot
- Custom display stuffs, for e.g. display of binary files and zip files
- Support multiple fonts
- Frame allocator; just use reference to growable allocator guy (maybe with some
  kind of exponential backoff-style thing?)
- Vim-style compile-time feature flags

## Cute but nope
- Customization?
- Client-server architecture so that we can have nice things
- Cross platform stuffs
- Custom display for zip files
- Custom display for raw binary, maybe using color for byte value in addition
  to hex representation?
- Error logging system
- Tiny lisp interpreter for syntax highlighting? Probably requires error logging
- C and Python APIs for scripting


## Cute but jesus christ no
- Language server support I guess?
- Full unicode support?
- Python-style indexable UTF-8 strings (just ascii vs full char probably)
- Abstract away graphics stuff with cute macros and stuff
- Cursor ref-counting or whatever works to get the behavior that Google docs has
- Terminal support

## Architecture
- Windows for rendering, tightly connected to graphics for each system
  - Windows have views into files that store stuff like where you cursor is
  - Window doesn't do anything by default. It only dispatches events, and when certain
    events come in, it runs those events
  - Views have a subset of the file data, and are not source of truth for anything
    except what to render.
- Files are managed globally, you call functions to modify the files and those
  functions might end up being IPC calls or whatever, to support multiple windows
- That way its easy to swap out an implementation with another, and also testing
  becomes a data-in/data-out problem instead of a mocking/contracts/whatever problem
