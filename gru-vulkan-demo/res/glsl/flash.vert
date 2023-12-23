#version 450

#include "cam.glsl"
#include "light.glsl"

layout (location=0) in vec3 position;
layout (location=1) in vec3 offset;
layout (location=2) in vec3 color;

layout (location=0) out vec3 col;
layout (location=1) out vec3 pos;
layout (location=2) out float brightness;

void main()
{
	col = color;
	pos = position + offset;
	brightness = exp(0.02 * ((position.z + offset.z) - light.z_bias));
	gl_Position = cam.proj * vec4(position + offset, 1.0);
}