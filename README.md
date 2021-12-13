# Editor
Text editor using WebGL and Rust/Wasm. Very much WIP.

## Feature Stuffs
- [ ] https://github.com/cmuratori/refterm
- [ ] https://webgl2fundamentals.org/webgl/lessons/webgl-qna-how-to-use-the-stencil-buffer.html
- [ ] https://medium.com/@evanwallace/easy-scalable-text-rendering-on-the-gpu-c3f4d782c5ac
- [ ] https://docs.rs/winit/latest/winit/
- [x] https://docs.rs/ttf-parser/latest/ttf_parser/
- [x] https://github.com/raphlinus/font-rs
- [x] https://webgl2fundamentals.org/webgl/lessons/webgl-how-it-works.html

## Size Opt Stuffs
- https://github.com/johnthagen/min-sized-rust
- https://www.skypack.dev/view/esbuild-plugin-wasm-pack

## Next Up
- Getting input and inserting into the editor
- Deleting text (BTree ig)
- Cursor stuff, probably some more architectural changes in the process
- Make text less ugly
- Background color stuff
- Line numbers
- Select text
- Window sizing and whatnot
- Command system for editor mutations

## Mid-term
- Find, Find and replace
- Multiple canvases on the web
- Syntax highlighting; probably just make something super duper simple
- Graphics/shader stuff
- Persist data and whatnot

## Far in the Future
- Customization?
- Client-server architecture so that we can have nice things
- Cross platform stuffs
- Abstract away graphics stuff with cute macros and stuff

