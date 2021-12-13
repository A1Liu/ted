import { start, render, newWebgl } from "./Cargo.toml";

// .visually-hidden {
//   position: absolute;
//   left:     -10000px;
//   top:      auto;
//   width:    1px;
//   height:   1px;
//   overflow: hidden;
// }

const timeout = (ms) => new Promise((res) => setTimeout(res, ms));

const repeat = async (func, ms = 1000, limit = 100) => {
  while (limit-- > 0) {
    func();
    await timeout(ms);
  }
};

let text = `Welcome to my stupid project to make a text editor.
And now, Kirin J. Callinan's "Big Enough":\n`;

start();
repeat(() => { render(text); text += 'aaah '; }).catch(console.warn);
