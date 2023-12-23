#version 450

#include "light.glsl"

layout (location=0) in vec3 color;
layout (location=1) in vec3 pos;
layout (location=2) in float brightness;

layout (location=0) out vec4 frag_color;

void main()
{
	vec3 pxl_to_cam = pos - light.pos;
	float distance = length(pxl_to_cam);
	vec3 pxl_to_cam_norm = pxl_to_cam / distance;
    distance *= 0.2;

	float lichtkegel = smoothstep(light.cos_angle_outer, light.cos_angle_inner, dot(pxl_to_cam_norm, light.dir)) * 5;
	float norm_stuff = clamp(1 / (1 + distance) / (1 + distance), 0, 1);
	
	frag_color.rgb = color * (lichtkegel * light.color * norm_stuff + light.ambient) * brightness + light.flash_ambient;
    frag_color.a = 1;
}