#version 450 core

layout (location=0) in vec2 position;
layout (location=1) in vec3 surface_coords;

layout (location=0) out vec3 tex_coords;

#include "cam.glsl"

void main()
{
    tex_coords = surface_coords;
    gl_Position = cam.proj * vec4(position, 0.0, 1.0);
}