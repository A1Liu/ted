uniform float width;
uniform float height;

attribute vec2 a_pos;

// WebGL 1 doesn't support integer attributes. Maybe someday we will use WebGL 2.
// attribute float glyph_in;

// varying float glyph;

void main() {
    vec4 temp_out = vec4(0.0, 0.0, 0.0, 1.0);

    float x = (a_pos.x / width) - 1.0;
    float y = -(a_pos.y / height) + 1.0;

    temp_out.x = x;
    temp_out.y = y;

    gl_Position = temp_out;
}
