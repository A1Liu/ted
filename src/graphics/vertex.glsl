#version 300 es

in uvec2 in_pos;
in uint in_block_type;
in uvec2 in_glyph_pos;

// TODO eventually split out text dimensions from clip space dimensions
uniform vec2 u_dims;

uniform vec2 u_atlas_dims;

out vec2 v_glyph_pos;
flat out uint v_block_type;

const vec2 c_pos_transform = vec2(1.0, -1.0);
const vec2 c_pos_offset = vec2(-1.0, 1.0);

/*
0       1,3



2,4     5
*/

void main() {
    // Translation from vertex ID to text coordinates
    int block_index = gl_VertexID % 6;
    int is_first = int(block_index < 3);
    float is_right = float(block_index % 2);
    float is_bot = float((block_index + is_first * 2) >= 4);
    vec2 block_offset = vec2(is_right, is_bot);

    int line_width = int(u_dims.x);
    int point_index = gl_VertexID / 6;
    int point_x = point_index % line_width;
    int point_y = point_index / line_width;

    // uvec2 in_pos = uvec2(point_x, point_y);

    // Translation from text coordinates to clip space
    vec2 pos = vec2(in_pos) / u_dims * 2.0;
    pos = pos * c_pos_transform + c_pos_offset;

    gl_Position = vec4(pos, 0.0, 1.0);
    v_glyph_pos = vec2(in_glyph_pos) / vec2(u_atlas_dims);
    v_block_type = in_block_type;
}
