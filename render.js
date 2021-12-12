import { render, newWebgl } from "./Cargo.toml";

// .visually-hidden {
//   position: absolute;
//   left:     -10000px;
//   top:      auto;
//   width:    1px;
//   height:   1px;
//   overflow: hidden;
// }


try {
  render();
} catch (e) {
  console.log(e);
}
