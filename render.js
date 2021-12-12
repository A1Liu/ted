import { render, newWebgl } from "./Cargo.toml";

// .visually-hidden {
//   position: absolute;
//   left:     -10000px;
//   top:      auto;
//   width:    1px;
//   height:   1px;
//   overflow: hidden;
// }

const webgl = newWebgl();

try {
  render(webgl);
} catch (e) {
  console.log(e);
}
