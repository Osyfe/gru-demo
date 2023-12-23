#version 450 core

#include "light.glsl"

layout (location=0) in vec3 pos;
layout (location=1) in vec3 normal;
layout (location=2) in vec2 tex_coords;
layout (location=3) in float brightness;

layout (set=2, binding=0) uniform sampler2D tex;

layout (location=0) out vec4 frag_color; 

vec3 light_factor()
{
	vec3 pxl_to_cam = pos - light.pos;
	float distance = length(pxl_to_cam);
	vec3 pxl_to_cam_norm = pxl_to_cam / distance;
    distance *= 0.2;

	float lichtkegel = smoothstep(light.cos_angle_outer, light.cos_angle_inner, dot(pxl_to_cam_norm, light.dir));
	float norm_stuff = clamp(dot(normal, -light.dir) / (1 + distance) / (1 + distance), 0, 1);
	return (lichtkegel * light.color * norm_stuff + light.ambient) * brightness;
}

void main()
{
    frag_color.rgb = texture(tex, tex_coords).rgb * light_factor() + light.flash_ambient;
    frag_color.a = 1;
}