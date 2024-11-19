use gru_wgpu::wgpu;
use gru_misc::math::{Vec3, Vec4, Mat4};
use super::warrior::Warrior;

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const UNIFORM_ALIGN: u64 = 256;

#[repr(C, packed)]
pub struct Vertex
{
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub tex_coords: [f32; 2]
}

#[repr(C)]
pub struct UniformVertex
{
    pub cam: Mat4
}

#[repr(C)]
pub struct UniformFragment
{
    pub cam_pos: Vec4,
    pub ambient_color: Vec4,
    pub sun_dir: Vec4,
    pub sun_color: Vec3,
    pub present: u32
}

pub struct Render
{
    warrior: Warrior,
    pipeline: wgpu::RenderPipeline,
    uniform_bind_group: wgpu::BindGroup,
    dynamic_buffer: wgpu::Buffer
}

impl Render
{
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat, texture_bind_group_layout: wgpu::BindGroupLayout, warrior: Warrior) -> Self
    {
        let (uniform_bind_group_layout, uniform_bind_group, dynamic_buffer) = dynamic_buffer(device);
        Self
        {
            warrior,
            pipeline: pipeline(device, surface_format, uniform_bind_group_layout, texture_bind_group_layout),
            uniform_bind_group,
            dynamic_buffer
        }
    }

    pub fn render(&self, queue: &wgpu::Queue, render_pass: &mut wgpu::RenderPass, uniform_v: UniformVertex, uniform_f: UniformFragment)
    {
        //uniform buffers
        let len_v = std::mem::size_of::<UniformVertex>();
        let len_f = std::mem::size_of::<UniformFragment>();
        let mut data = vec![0; UNIFORM_ALIGN as usize + len_f];
        let uniform_v = [uniform_v];
        let data_v = unsafe { std::slice::from_raw_parts(uniform_v.as_ptr() as *const u8, std::mem::size_of::<UniformVertex>()) };
        let uniform_f = [uniform_f];
        let data_f = unsafe { std::slice::from_raw_parts(uniform_f.as_ptr() as *const u8, std::mem::size_of::<UniformFragment>()) };
        data[0..len_v].copy_from_slice(data_v);
        data[(UNIFORM_ALIGN as usize)..].copy_from_slice(data_f);
        queue.write_buffer(&self.dynamic_buffer, 0, &data);

        //render warrior
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.warrior.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.warrior.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        for mesh in &self.warrior.meshes
        {
            render_pass.set_bind_group(1, &mesh.texture_bind_group, &[]);
            render_pass.draw_indexed(mesh.indices.clone(), 0, 0..1);
        }
    }
}

fn pipeline(device: &wgpu::Device, surface_format: wgpu::TextureFormat, uniform_bind_group_layout: wgpu::BindGroupLayout, texture_bind_group_layout: wgpu::BindGroupLayout) -> wgpu::RenderPipeline
{
    let pipeline_layout_descr = wgpu::PipelineLayoutDescriptor
    {
        label: None,
        bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
        push_constant_ranges: &[]
    };
    let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_descr);

    let shader_descr = wgpu::include_wgsl!("../wgsl/warrior.wgsl");
    let shader = device.create_shader_module(shader_descr);
    let color_target_state = Some(wgpu::ColorTargetState
    {
        format: surface_format,
        blend: None,
        write_mask: wgpu::ColorWrites::all()
    });
    let pipeline_descr = wgpu::RenderPipelineDescriptor
    {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState
        {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers:
            &[
                wgpu::VertexBufferLayout
                {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes:
                    &[
                        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: std::mem::offset_of!(Vertex, position) as u64, shader_location: 0 },
                        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: std::mem::offset_of!(Vertex, normal) as u64, shader_location: 1 },
                        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: std::mem::offset_of!(Vertex, tangent) as u64, shader_location: 2 },
                        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: std::mem::offset_of!(Vertex, tex_coords) as u64, shader_location: 3 }
                    ]
                }
            ]
        },
        primitive: wgpu::PrimitiveState
        {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false
        },
        depth_stencil: Some(wgpu::DepthStencilState
        {
            format: DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState
            {
                front: wgpu::StencilFaceState
                {
                    compare: wgpu::CompareFunction::Never,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Keep
                },
                back: wgpu::StencilFaceState
                {
                    compare: wgpu::CompareFunction::Never,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Keep
                },
                read_mask: 0,
                write_mask: 0
            },
            bias: wgpu::DepthBiasState { constant: 0, slope_scale: 0.0, clamp: 0.0 }
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState
        {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: std::slice::from_ref(&color_target_state)
        }),
        multiview: None,
        cache: None
    };
    device.create_render_pipeline(&pipeline_descr)
}

fn dynamic_buffer(device: &wgpu::Device) -> (wgpu::BindGroupLayout, wgpu::BindGroup, wgpu::Buffer)
{
    let len_v = std::mem::size_of::<UniformVertex>();
    assert!((len_v as u64) < UNIFORM_ALIGN, "expand uniform align logic...");
    let len_f = std::mem::size_of::<UniformFragment>();
    let buffer_descr = wgpu::BufferDescriptor
    {
        label: None,
        size: UNIFORM_ALIGN + len_f as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        mapped_at_creation: false
    };
    let buffer = device.create_buffer(&buffer_descr);

    let bind_group_layout_descr = wgpu::BindGroupLayoutDescriptor
    {
        label: None,
        entries:
        &[
            wgpu::BindGroupLayoutEntry
            {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer
                {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None
            },
            wgpu::BindGroupLayoutEntry
            {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer
                {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None
                },
                count: None
            }
        ]
    };
    let bind_group_layout = device.create_bind_group_layout(&bind_group_layout_descr);
    let bind_group_descr = wgpu::BindGroupDescriptor
    {
        label: None,
        layout: &bind_group_layout,
        entries:
        &[
            wgpu::BindGroupEntry
            {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding
                {
                    buffer: &buffer,
                    offset: 0,
                    size: Some(wgpu::BufferSize::new(len_v as u64).unwrap())
                })
            },
            wgpu::BindGroupEntry
            {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding
                {
                    buffer: &buffer,
                    offset: UNIFORM_ALIGN as wgpu::BufferAddress,
                    size: Some(wgpu::BufferSize::new(len_f as u64).unwrap())
                })
            }
        ]
    };
    let bind_group = device.create_bind_group(&bind_group_descr);

    (bind_group_layout, bind_group, buffer)
}

pub fn texture_bind_group_layout(device: &wgpu::Device) -> (wgpu::BindGroupLayout, wgpu::Sampler)
{
    let bind_group_layout_descr = wgpu::BindGroupLayoutDescriptor
    {
        label: None,
        entries:
        &[
            wgpu::BindGroupLayoutEntry
            {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture
                {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false
                },
                count: None
            },
            wgpu::BindGroupLayoutEntry
            {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture
                {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false
                },
                count: None
            },
            wgpu::BindGroupLayoutEntry
            {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture
                {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false
                },
                count: None
            },
            wgpu::BindGroupLayoutEntry
            {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None
            }
        ]
    };
    let bind_group_layout = device.create_bind_group_layout(&bind_group_layout_descr);
    let sampler_descr = wgpu::SamplerDescriptor
    {
        label: None,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        lod_min_clamp: 0.0,
        lod_max_clamp: 32.0,
        compare: None,
        anisotropy_clamp: 1,
        border_color: None
    };
    let sampler = device.create_sampler(&sampler_descr);
    (bind_group_layout, sampler)
}

pub fn texture_bind_group(device: &wgpu::Device, textures: [&wgpu::Texture; 3], texture_bind_group_layout: &wgpu::BindGroupLayout, sampler: &wgpu::Sampler) -> wgpu::BindGroup
{
    let texture_view_descr = wgpu::TextureViewDescriptor
    {
        label: None,
        format: None,
        dimension: Some(wgpu::TextureViewDimension::D2),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None
    };
    let texture_views =
    [
        textures[0].create_view(&texture_view_descr),
        textures[1].create_view(&texture_view_descr),
        textures[2].create_view(&texture_view_descr)
    ];
    let bind_group_descr = wgpu::BindGroupDescriptor
    {
        label: None,
        layout: texture_bind_group_layout,
        entries:
        &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&texture_views[0]) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&texture_views[1]) },
            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&texture_views[2]) },
            wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&sampler) }
        ]
    };
    let bind_group = device.create_bind_group(&bind_group_descr);
    bind_group
}
