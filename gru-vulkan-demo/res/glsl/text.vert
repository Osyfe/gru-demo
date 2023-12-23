#version 450 core

#include "text.glsl"

layout (location=0) in vec2 position;
layout (location=1) in vec3 tex_coords;

layout (location=0) out vec3 coords;

void main()
{
	coords = tex_coords;
	gl_Position = vec4(position * text.height / vec2(text.aspect, 1.0) + vec2(-1.0, 1.0 - text.height), 0.0, 1.0);
}