//vertex

struct VSInput
{
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec3<f32>,
    @location(3) tex_coords: vec2<f32>,
};

struct DynamicVertex
{
    cam: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> dynamic_v: DynamicVertex;

@vertex
fn vs_main(in: VSInput) -> VSOutput
{
    var bitangent = cross(in.normal, in.tangent);
    return VSOutput
    (
        dynamic_v.cam * vec4<f32>(in.position.xyz, 1.0),
        in.position,
        in.normal,
        in.tangent,
        bitangent,
        in.tex_coords,
    );
}

//both

struct VSOutput
{
    @builtin(position) pos: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec3<f32>,
    @location(3) bitangent: vec3<f32>,
    @location(4) coords: vec2<f32>,
}

//fragment

const FULL: u32 = 0;
const NORMAL: u32 = 1;
const ROUGHNESS: u32 = 2;

struct DynamicFragment
{
    cam_pos: vec3<f32>,
    ambient_color: vec3<f32>,
    sun_dir: vec3<f32>,
    sun_color: vec3<f32>,
    present: u32
}

@group(0) @binding(1) var<uniform> dynamic_f: DynamicFragment;
@group(1) @binding(0) var diffuse: texture_2d<f32>;
@group(1) @binding(1) var normal: texture_2d<f32>;
@group(1) @binding(2) var roughness: texture_2d<f32>;
@group(1) @binding(3) var tex_sampler: sampler;

fn srgb2rgb(srgb: f32) -> f32
{
    if srgb <= 0.04045 { return srgb / 12.92; }
    else { return pow((srgb + 0.055) / 1.055, 2.4); }
}

fn rough_to_shiny(roughness: f32) -> f32
{
    return pow((1.0 - roughness), 2.0) * 14.0 + 1.0;
}

const PI: f32 = 3.1415926538;

fn light_factor(k: vec3<f32>, n: vec3<f32>, l: vec3<f32>, nu: f32) -> f32
{
    var a = dot(k, n);
    var b = 1.0 - a;
    var a0 = dot(l, n);
    var c = 1.0 - a0;
    if a0 <= 0.0 || a <= 0.0 { return 0.0; }
    var norm = a0 * (nu + 1.0) / (2.0 * PI);
    if a >= 1.0 { return norm * pow(a0, nu); }
    if a0 >= 1.0 { return norm * pow(a, nu); }
    var mbc = (dot(k, l) - a*a0) * sqrt(b * c / ((a + 1.0) * (a0 + 1.0)));
    var b2 = b*b;
    var c2 = c*c;
    return norm * pow(1.0 - sqrt(max(0.0, b2 + 2.0*mbc + c2) / max(0.001, 1.0 + 2.0*mbc + b2*c2)), nu);
}

@fragment
fn fs_main(in: VSOutput) -> @location(0) vec4<f32>
{
    var tex_diffuse = textureSample(diffuse, tex_sampler, in.coords);
    if tex_diffuse.a < 0.5 { discard; }
    var tex_normal = normalize(textureSample(normal, tex_sampler, in.coords).rgb - vec3<f32>(0.5));
    var tex_roughness = textureSample(roughness, tex_sampler, in.coords).r;
   
    var world_normal_unit = normalize(in.normal);
    var world_normal_local = tex_normal.r * normalize(in.tangent) + tex_normal.g * normalize(in.bitangent) + tex_normal.b * world_normal_unit;
    var camera_unit = normalize(dynamic_f.cam_pos - in.position);

    var frag_color = vec4<f32>(dynamic_f.ambient_color, 1.0);

    var light_direction_unit = normalize(dynamic_f.sun_dir);
    frag_color += vec4<f32>(dynamic_f.sun_color * light_factor(camera_unit, world_normal_local, light_direction_unit, rough_to_shiny(tex_roughness)), 0.0);

    frag_color *= tex_diffuse;

    switch dynamic_f.present
    {
        case FULL { return frag_color; }
        case NORMAL { return vec4<f32>(world_normal_local * vec3<f32>(0.5) + vec3<f32>(0.5), 1.0); }
        case ROUGHNESS { return vec4<f32>(1.0, srgb2rgb(tex_roughness), 1.0, 1.0); }
        default { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
    }
}
