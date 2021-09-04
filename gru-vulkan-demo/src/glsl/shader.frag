#version 450 core

layout (location=0) in vec3 tex_coords;

layout (location=0) out vec4 frag_color;

layout (set=0, binding=1) uniform sampler2DArray tex;

void main()
{
	float alpha = texture(tex, tex_coords).r;
    frag_color = vec4(0.0, 0.1, 0.8, alpha);
}
