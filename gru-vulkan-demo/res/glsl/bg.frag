#version 450 core

#include "light.glsl"

layout (location=0) in float z;

layout (location=0) out vec4 frag_color;

void main()
{
    float brightness = exp(0.02 * (z - light.z_bias));
	frag_color.rgb = vec3(0.05, 0.05, 0.05) * brightness * light.ambient;
	frag_color.a = 1.0;
}