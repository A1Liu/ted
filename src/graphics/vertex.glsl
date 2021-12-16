#version 300 es

in uvec3 in_pos;
in uvec2 in_glyph_pos;

uniform vec2 u_dims;
uniform vec2 u_atlas_dims;

out vec2 v_glyph_pos;
flat out uint v_block_kind;

const vec2 c_pos_transform = vec2(1.0, -1.0);
const vec2 c_pos_offset = vec2(-1.0, 1.0);

void main() {
    vec2 pos = vec2(in_pos.xy) / u_dims * 2.0;
    pos = pos * c_pos_transform + c_pos_offset;

    gl_Position = vec4(pos, 0.0, 1.0);
    v_glyph_pos = vec2(in_glyph_pos) / vec2(u_atlas_dims);
    v_block_kind = uint(in_pos.z);
}
