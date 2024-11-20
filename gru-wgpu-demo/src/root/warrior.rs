use gru_wgpu::{file, wgpu::{self, util::DeviceExt}};
use gru_misc::{futures::*, color::Color, image, gltf, file_tree::*};
use std::{sync::Arc, ops::Range};
use super::render::{self, Vertex};

fn float_to_u8(f: f32) -> u8
{
    (f.max(0.0).min(1.0) * 256.0) as u8
}

fn create_texture(device: &wgpu::Device, queue: &wgpu::Queue, data: image::Image, format: wgpu::TextureFormat) -> wgpu::Texture
{
    let image::Image { width, height, channels: _, data } = data;
    let texture_descr = wgpu::TextureDescriptor
    {
        label: None,
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };
    device.create_texture_with_data(&queue, &texture_descr, wgpu::util::TextureDataOrder::LayerMajor, &data)
}

pub struct Mesh
{
    pub texture_bind_group: wgpu::BindGroup,
    pub indices: Range<u32>
}

pub struct Warrior
{
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub meshes: Vec<Mesh>,
    _textures: ahash::AHashMap<String, wgpu::Texture>
}

impl Warrior
{
    pub async fn load(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> (Self, wgpu::BindGroupLayout)
    {
        let model = Self::load_model(&device);
        let textures = Self::load_textures(&device, &queue);
        let ((vertex_buffer, index_buffer, mut meshes), mut textures) = join!(model, textures).await;
        //generate missing textures from consts
        for mesh in &mut meshes
        {
            if let gltf::TextureOrConstant::Constant(color) = &mesh.diffuse_texture
            {
                let name = format!("{}_diffuse_synth.png", mesh.name);
                let color = Color::from_normalized_linear(color[0], color[1], color[2], color[3]);
                let color = color.to_normalized_srgb();
                let color = [color.0, color.1, color.2, color.3];
                let data = image::Image { width: 1, height: 1, channels: 4, data: color.into_iter().map(float_to_u8).collect() };
                let texture = create_texture(&device, &queue, data, wgpu::TextureFormat::Rgba8UnormSrgb);
                textures.insert(name.clone(), texture);
                mesh.diffuse_texture = gltf::TextureOrConstant::Texture(name);
            }
            if let None = &mesh.normal_texture
            {
                let name = format!("{}_normal_synth.png", mesh.name);
                let data = image::Image { width: 1, height: 1, channels: 4, data: [0.5, 0.5, 1.0, 1.0].into_iter().map(float_to_u8).collect() };
                let texture = create_texture(&device, &queue, data, wgpu::TextureFormat::Rgba8Unorm);
                textures.insert(name.clone(), texture);
                mesh.normal_texture = Some(name);
            }
            if let gltf::TextureOrConstant::Constant(color) = &mesh.roughness_texture
            {
                let name = format!("{}_roughness_synth.png", mesh.name);
                let data = image::Image { width: 1, height: 1, channels: 1, data: vec![float_to_u8(color[0])] };
                let texture = create_texture(&device, &queue, data, wgpu::TextureFormat::R8Unorm);
                textures.insert(name.clone(), texture);
                mesh.roughness_texture = gltf::TextureOrConstant::Texture(name);
            }
            yield_now().await;
        }
        //link textures to meshes
        let (bind_group_layout, sampler) = render::texture_bind_group_layout(&device);
        yield_now().await;
        let meshes = meshes.into_iter().map(|mesh|
        {
            let diffuse = match mesh.diffuse_texture
            {
                gltf::TextureOrConstant::Texture(name) => textures.get(name.as_str()).expect(&format!("missing texture \"{name}\"")),
                gltf::TextureOrConstant::Constant(_) => unreachable!()
            };
            let normal = match mesh.normal_texture
            {
                Some(name) => textures.get(name.as_str()).expect(&format!("missing texture \"{name}\"")),
                None => unreachable!()
            };
            let roughness = match mesh.roughness_texture
            {
                gltf::TextureOrConstant::Texture(name) => textures.get(name.as_str()).expect(&format!("missing texture \"{name}\"")),
                gltf::TextureOrConstant::Constant(_) => unreachable!()
            };
            let textures = [diffuse, normal, roughness];
            let texture_bind_group = render::texture_bind_group(&device, textures, &bind_group_layout, &sampler);
            let indices = (mesh.indices.start as u32)..(mesh.indices.end as u32);
            Mesh { texture_bind_group, indices }
        }).collect();
        (Self { vertex_buffer, index_buffer, meshes, _textures: textures }, bind_group_layout)
    }

    async fn load_model(device: &Arc<wgpu::Device>) -> (wgpu::Buffer, wgpu::Buffer, Vec<gltf::Mesh>)
    {
        let mut loader = file::Loader::new();
        let gltf = loader.load("data/warrior/warrior.gltf");
        let bin = loader.load("data/warrior/warrior.bin");
        let (gltf, bin) = join!(gltf, bin).await;
        let (gltf, bin) = (gltf.unwrap(), bin.unwrap());
        let mut model = gltf::Model::decode(&gltf, &bin);
        let mut vertices = Vec::with_capacity(model.positions.len());
        for (((position, normal), tangent), tex_coords) in model.positions.into_iter().zip(model.normals.unwrap()).zip(model.tangents.unwrap()).zip(model.tex_coords.unwrap())
        {
            let vertex = Vertex
            {
                position: position.into(),
                normal: normal.into(),
                tangent: tangent.into(),
                tex_coords: tex_coords.into()
            };
            vertices.push(vertex);
        }

        //patch indices (WebGL does not support base_vertex)
        yield_now().await;
        for mesh in &model.meshes
        {
            let base_vertex = mesh.vertices.start as u32;
            for index in &mut model.indices[mesh.indices.clone()] { *index += base_vertex; }
        }

        yield_now().await;
        let data = unsafe { std::slice::from_raw_parts(vertices.as_ptr() as *const u8, vertices.len() * std::mem::size_of::<Vertex>()) };
        let buffer_descr = wgpu::util::BufferInitDescriptor
        {
            label: None,
            contents: data,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX
        };
        let vertex_buffer = device.create_buffer_init(&buffer_descr);
        let data = unsafe { std::slice::from_raw_parts(model.indices.as_ptr() as *const u8, model.indices.len() * 4) };
        let buffer_descr = wgpu::util::BufferInitDescriptor
        {
            label: None,
            contents: data,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX
        };
        let index_buffer = device.create_buffer_init(&buffer_descr);
        (vertex_buffer, index_buffer, model.meshes)
    }

    async fn load_textures(device: &Arc<wgpu::Device>, queue: &Arc<wgpu::Queue>) -> ahash::AHashMap<String, wgpu::Texture>
    {
        let mut loader = file::Loader::new();
        let list = tree!("gru-wgpu-demo/export/data/warrior/textures", "data/warrior/textures");
        let iter = list.files.iter().map(|file|
        {
            let format =
                if file.name.contains("normal") { wgpu::TextureFormat::Rgba8Unorm }
                else if file.name.contains("roughness") { wgpu::TextureFormat::R8Unorm }
                else
                {
                    if !file.name.contains("diffuse") { log::warn!("interpreted \"{}\" as diffuse texture", file.name); }
                    wgpu::TextureFormat::Rgba8UnormSrgb
                };
            async fn texture(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, file: file::File, format: wgpu::TextureFormat) -> wgpu::Texture
            {
                let img = file.await.unwrap();
                let mut data = image::Image::decode(&img, image::Config::new(image::Format::Png));
                //roughness texture but loaded 4 channels -> extract green channel (lol)
                if format == wgpu::TextureFormat::R8Unorm { data.extract_channel(1); }
                create_texture(&device, &queue, data, format)
            }
            texture(device.clone(), queue.clone(), loader.load(file.path), format)
        });
        let textures = join_all(iter).await;
        let mut map = ahash::AHashMap::with_capacity(list.files.len());
        for (file, texture) in list.files.iter().zip(textures) { map.insert(file.name.to_owned(), texture); }
        map
    }
}
