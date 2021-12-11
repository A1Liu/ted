import { render, test_print } from "./Cargo.toml";

test_print();

const canvas = document.getElementById("canvas");

try {
  render(canvas);
} catch (e) {
  console.log(e);
}
