#version 450 core

#include "cam.glsl"
#include "light.glsl"

layout (location=0) in vec3 position;
layout (location=1) in vec3 normal;
layout (location=2) in vec2 coords;

layout (location=0) out vec3 pos;
layout (location=1) out vec3 normal_out;
layout (location=2) out vec2 tex_coords;
layout (location=3) out float brightness;

void main()
{
	pos = position;
	normal_out = normal;
	tex_coords = coords;
	brightness = exp(0.02 * (position.z - light.z_bias));
    gl_Position = cam.proj * vec4(position, 1.0);
}