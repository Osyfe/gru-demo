attribute vec3 position;
attribute vec3 color;
attribute vec2 tex_coords;

uniform mat4 mat;

varying vec2 coords;
varying vec3 col;

void main()
{
	col = color;
	coords = tex_coords;
    gl_Position = mat * vec4(position, 1.0);
}
