varying vec2 coords;

uniform sampler2D atlas;

void main()
{
	float alpha = texture2D(atlas, coords).a;
	gl_FragColor = vec4(vec3(0.0), alpha);
}
