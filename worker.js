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
const fontSizeText = window.getComputedStyle(canvas, null).getPropertyValue('font-size');
const fontSize = parseFloat(fontSizeText);

const ctx = canvas.getContext("webgl2", {
  premultipliedAlpha: false,
});

try {
  render(ctx);
} catch (e) {
  console.log(e);
}
