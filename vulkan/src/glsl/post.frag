#version 450

layout(location=0) in vec2 coords;

layout (set=0, binding=0) uniform sampler2D depth;
layout (input_attachment_index=0, set=0, binding=1) uniform subpassInput color;

layout (location=0) out vec4 frag_color;

void main()
{
	frag_color = subpassLoad(color) * texture(depth, coords).r;
}