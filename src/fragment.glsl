#version 300 es
precision mediump float;

in vec2 glyph_pos;

out vec4 out_color; // you can pick any name

void main() {
    vec2 pos = glyph_pos - vec2(0, 1267.0);
    pos /= 300.0;

    out_color = vec4(pos, 0.5, 1.0);
}
