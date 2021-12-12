import { render, test_print } from "./Cargo.toml";

// .visually-hidden {
//   position: absolute;
//   left:     -10000px;
//   top:      auto;
//   width:    1px;
//   height:   1px;
//   overflow: hidden;
// }

const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("webgl2", {
  premultipliedAlpha: false,
});

try {
  render(ctx);
} catch (e) {
  console.log(e);
}
