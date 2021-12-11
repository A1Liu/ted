#version 300 es
precision mediump float;

in vec2 v_glyph_pos;

uniform sampler2D u_glyph_atlas;

out vec4 out_color; // you can pick any name

void main() {
    vec4 color = texture(u_glyph_atlas, v_glyph_pos);
    // color.x = 0.5;
    // color.y = 0.5;
    // color.z = 0.5;
    // color.w = 1.0;

    out_color = color;
}
