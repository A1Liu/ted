#version 300 es
precision mediump float;

in vec2 v_glyph_pos;
in vec3 v_fg_color;
in vec3 v_bg_color;

uniform sampler2D u_glyph_atlas;

out vec4 out_color; // you can pick any name

void main() {
    vec4 in_fg = vec4(v_fg_color, 1.0);
    vec4 in_bg = vec4(v_bg_color, 1.0);

    float text_px = texture(u_glyph_atlas, v_glyph_pos).x;

    vec4 fg_color = text_px * in_fg;
    vec4 bg_color = in_bg + (text_px * -in_bg);

    out_color = bg_color + fg_color;
}
