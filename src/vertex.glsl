uniform float width;
uniform float height;

attribute uvec2 position;
// attribute uint glyph_in;

// varying uint glyph;

void main() {
    vec4 coordinates = vec4(0.0, 0.0, 0.0, 0.0);

    float x = float(position.x) / width;
    float y = float(position.y) / height;

    coordinates.x = x;
    coordinates.y = y;

    gl_Position = coordinates;
}
