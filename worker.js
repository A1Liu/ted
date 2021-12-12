import { render, test_print } from "./Cargo.toml";

test_print();

const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("webgl2", {
  premultipliedAlpha: false,
});

try {
  render(ctx);
} catch (e) {
  console.log(e);
}
