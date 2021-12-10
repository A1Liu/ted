// uniform float width;
// uniform float height;

attribute vec4 a_pos;
// attribute uint glyph_in;

// varying uint glyph;

void main() {
    // vec4 temp_out = vec4(coordinates, 0.0);

    // float x = coordinates.x;
    // float y = coordinates.y;

    // temp_out.x = x;
    // temp_out.y = y;

    gl_Position = a_pos;
}
