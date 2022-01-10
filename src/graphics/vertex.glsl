#version 300 es

in uvec2 in_glyph_pos;
in vec3 in_fg_color;
in vec3 in_bg_color;

// TODO eventually split out text dimensions from clip space dimensions
uniform vec2 u_dims;

uniform float u_clip_begin;
uniform float u_clip_end;
uniform vec2 u_atlas_dims;

out vec3 v_fg_color;
out vec3 v_bg_color;
out vec2 v_glyph_pos;

/*
    0       1,3



    2,4     5
*/

// This was a lot simpler when I could use integer division. :(
void main() {
    // Translation from vertex ID to text coordinates
    float vertex_id = float(gl_VertexID);
    float block_index = floor(mod(vertex_id, 6.));
    float point_index = floor(vertex_id / 6.);

    float is_first = float(block_index < 3.);
    float is_right = floor(mod(block_index, 2.));
    float is_bot = float((block_index + is_first * 2.) >= 4.);
    vec2 block_offset = vec2(is_right, is_bot);

    // Floating point precision means that removing these 0.001 offsets results
    // in weird stuff. I don't understand why.
    float point_x = floor(mod(point_index + 0.001, u_dims.x));
    float point_y = floor(point_index / u_dims.x + 0.001);

    vec2 in_pos = vec2(point_x, point_y) + block_offset;

    // Translation from text coordinates to clip space
    float clip_len = u_clip_end - u_clip_begin;
    vec2 pos = in_pos / u_dims * vec2(clip_len, -2.0) + vec2(u_clip_begin, 1.0);

    gl_Position = vec4(pos, 0.0, 1.0);
    v_glyph_pos = vec2(in_glyph_pos) / vec2(u_atlas_dims);
    v_fg_color = in_fg_color;
    v_bg_color = in_bg_color;
}
