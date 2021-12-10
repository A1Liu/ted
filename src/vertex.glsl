#version 300 es

in uvec2 in_pos;

uniform float u_width;
uniform float u_height;

// attribute float glyph_in;

// varying float glyph;

void main() {
    vec4 temp_out = vec4(0.0, 0.0, 0.0, 1.0);

    vec2 pos = vec2(in_pos);
    float width = u_width / 2.0;
    float height = u_height / 2.0;

    float x = (pos.x / width) - 1.0;
    float y = -(pos.y / height) + 1.0;

    temp_out.x = x;
    temp_out.y = y;

    gl_Position = temp_out;
}
