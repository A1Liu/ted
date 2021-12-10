import { render } from "./Cargo.toml";

const canvas = document.getElementById("canvas");
try {
  render(canvas);
} catch (e) {
  console.log(e);
}
