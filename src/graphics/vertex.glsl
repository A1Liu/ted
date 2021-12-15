#version 300 es

in uvec2 in_pos;
in uvec2 in_glyph_pos;

uniform uvec2 u_cursor_pos;

uniform float u_width;
uniform float u_height;

uniform uint u_atlas_width;
uniform uint u_atlas_height;

out vec2 v_glyph_pos;

out float v_is_cursor;

void main() {
    vec4 temp_out = vec4(0.0, 0.0, 0.0, 1.0);

    vec2 pos = vec2(in_pos);
    float width = u_width / 2.0;
    float height = u_height / 2.0;

    float x = (pos.x / width) - 1.0;
    float y = -(pos.y / height) + 1.0;

    temp_out.x = x;
    temp_out.y = y;

    uvec2 atlas_dims = uvec2(u_atlas_width, u_atlas_height);

    vec2 temp_glyph_out = vec2(in_glyph_pos) / vec2(atlas_dims);

    bool is_cursor = u_cursor_pos == in_pos;

    gl_Position = temp_out;
    v_glyph_pos = temp_glyph_out;
    v_is_cursor = float(is_cursor);
}
