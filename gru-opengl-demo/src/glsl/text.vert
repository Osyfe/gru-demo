attribute vec2 position;
attribute vec2 tex_coords;

uniform float height;

varying vec2 coords;

void main()
{
	coords = tex_coords;
	gl_Position = vec4(position * height + vec2(-1.0, 1.0), 0.0, 1.0);
}
