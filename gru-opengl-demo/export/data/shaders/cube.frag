varying vec3 col;
varying vec2 coords;

uniform sampler2D tex;

void main()
{
    gl_FragColor = vec4((1.0 - texture2D(tex, coords).r) * col, 1.0);
}
