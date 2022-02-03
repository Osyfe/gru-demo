varying vec2 coords;

uniform sampler2D atlas;

void main()
{
	gl_FragColor = vec4(texture2D(atlas, coords).rgb, 1.0);
}
