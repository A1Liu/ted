#version 300 es

in uvec2 in_pos;
in uint in_block_type;
in uvec2 in_glyph_pos;

uniform vec2 u_dims;
uniform vec2 u_atlas_dims;

out vec2 v_glyph_pos;
flat out uint v_block_type;

const vec2 c_pos_transform = vec2(1.0, -1.0);
const vec2 c_pos_offset = vec2(-1.0, 1.0);

void main() {
    // uint vertex_index = uint(gl_VertexID);
    // uint point_index = vertex_index / 6u;
    // uint block_index = vertex_index / 6u;

    vec2 pos = vec2(in_pos) / u_dims * 2.0;
    pos = pos * c_pos_transform + c_pos_offset;

    gl_Position = vec4(pos, 0.0, 1.0);
    v_glyph_pos = vec2(in_glyph_pos) / vec2(u_atlas_dims);
    v_block_type = in_block_type;
}
