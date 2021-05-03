varying vec2 coords;

uniform sampler2D atlas;

void main()
{
	gl_FragColor.r = texture2D(atlas, coords).a;
}
