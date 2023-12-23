layout (std140, set=1, binding=0) uniform Light
{
    float z_bias;
    float cos_angle_inner;
    float cos_angle_outer;
    vec3 ambient;
    vec3 flash_ambient;
    vec3 color;
    vec3 pos;
    vec3 dir;
} light;