#version 300 es
precision mediump float;

in vec2 v_glyph_pos;

// see src/graphics/mod.rs for more info on v_block_type
flat in uint v_block_type;

uniform sampler2D u_glyph_atlas;

out vec4 out_color; // you can pick any name

void main() {
    vec4 in_bg = vec4(0.3, 0.3, 0.3, 1.0);
    vec4 in_cursor = vec4(1.0, 1.0, 1.0, 1.0);
    vec4 in_fg = vec4(0.4, 0.4, 1.0, 1.0);

    float text_px = texture(u_glyph_atlas, v_glyph_pos).x;

    if (v_block_type == 0u) {
        vec4 fg_color = text_px * in_fg;
        vec4 bg_color = in_bg + (text_px * -in_bg);

        out_color = bg_color + fg_color;
    } else if (v_block_type == 1u) {
        vec4 fg_color = in_cursor + (text_px * -in_cursor);
        vec4 bg_color = text_px * in_bg;

        out_color = bg_color + fg_color;
    } else if (v_block_type == 2u) {
        out_color = vec4(0.9, 0.1, 1.0, 1.0);
    } else {
        out_color = vec4(0.9, 0.1, 1.0, 1.0);
    }
}
