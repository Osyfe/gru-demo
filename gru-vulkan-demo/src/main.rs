mod cave;
mod camera;
mod marching_cubes;
mod consts;
mod flash;

use gru_vulkan::*;
use gru_misc::{math::*, text_sdf::*, time::*};
use winit::{*, event_loop::ControlFlow, event::{VirtualKeyCode, ElementState}};
use noise::{self, Seedable, NoiseFn};
use std::{sync::{mpsc, Arc, Mutex}, collections::{HashSet, HashMap, hash_map}};

const ATLAS_SIZE: u32 = 512;

const CAVE_VERTEX: Shader = vert_shader!("res/glsl/cave.vert");
const CAVE_FRAGMENT: Shader = frag_shader!("res/glsl/cave.frag");
const FLASH_VERTEX: Shader = vert_shader!("res/glsl/flash.vert");
const FLASH_FRAGMENT: Shader = frag_shader!("res/glsl/flash.frag");
const BG_VERTEX: Shader = vert_shader!("res/glsl/bg.vert");
const BG_FRAGMENT: Shader = frag_shader!("res/glsl/bg.frag");
const TEXT_VERTEX: Shader = vert_shader!("res/glsl/text.vert");
const TEXT_FRAGMENT: Shader = frag_shader!("res/glsl/text.frag");

#[derive(Clone, Copy, DescriptorStructReprC)]
#[repr(C)]
pub struct CamBinding
{
    mat: Mat4,
}

#[derive(Clone, Copy, DescriptorStructReprC)]
#[repr(C)]
pub struct LightBinding
{
    z_bias: f32,
    cos_angle_inner: f32,
    cos_angle_outer: f32,
    _padding0: f32,
    ambient: (f32, f32, f32),
    _padding1: f32,
    flash_ambient: (f32, f32, f32),
    _padding2: f32,
    color: (f32, f32, f32),
    _padding3: f32,
    pos: (f32, f32, f32),
    _padding4: f32,
    dir: (f32, f32, f32)
}

#[derive(Clone, Copy, DescriptorStructReprC)]
#[repr(C)]
pub struct TextBinding
{
    aspect: f32,
    height: f32
}

#[derive(VertexAttributeGroupReprCpacked)]
#[repr(C, packed)]
pub struct Vertex
{
    #[location = 0]
    pub position: F3,
    #[location = 1]
    pub normal: F3,
    #[location = 2]
    pub tex_coords: F2
}

#[derive(VertexAttributeGroupReprCpacked)]
#[repr(C, packed)]
pub struct TextVertex
{
    #[location = 0]
    pub position: F2,
    #[location = 1]
    pub tex_coords: F3
}

fn main()
{
//window setup
    let mut event_loop = event_loop::EventLoop::new();
    let window = window::WindowBuilder::new()
        .with_title("gru_vulkan_demo: Cave Jumper")
        .with_inner_size(dpi::PhysicalSize { width: 1024.0_f32, height: 768.0 })
        .with_visible(false)
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();
    window.set_cursor_visible(false);
    //let monitor = window.available_monitors().nth(0).unwrap();
    //let mode = monitor.video_modes().nth(0).unwrap();
    //window.set_fullscreen(Some(window::Fullscreen::Exclusive(mode)));
    //window.set_fullscreen(Some(window::Fullscreen::Borderless(None)));
    let (width, height) = window.inner_size().into();
//initialization, queue fetching and swapchain creation
    let instance = Instance::new(Some(&window));
    let physical_devices = instance.physical_devices();
    let geforce = &physical_devices[0];
    let graphic_queue_family_info = &geforce.queue_families()[0];
    let device = instance.logical_device(geforce, vec![(graphic_queue_family_info, vec![1.0])]);
    let graphic_queue_family = device.get_queue_family(graphic_queue_family_info);
    let graphic_queue = graphic_queue_family.get_queue(0);
    let command_pool = device.new_command_pool(graphic_queue_family);
    let swapchain = device.new_swapchain(None, true).unwrap();
//texture
    let (image_type, texture, sampler) =
    {
        let img = image::io::Reader::open("data/rocks.png").unwrap().decode().unwrap().to_bgra8();
        let (tex_width, tex_height) = img.dimensions();
        let image_type = ImageType { channel: ImageChannelType::BgraSrgb, width: tex_width, height: tex_height, layers: ImageLayers::Single };
        let mut stage_buffer = device.new_image_buffer(image_type);
        stage_buffer.write(&img);
        let texture = device.new_image(image_type, ImageUsage::Texture { mipmapping: true });
        command_pool.new_command_buffer().copy_to_image(&graphic_queue.lock().unwrap(), &stage_buffer, &texture, 0, device.new_fence(false)).mark.wait();
        let sampler = device.new_sampler(SamplerInfo
        {
            min_filter: SamplerFilter::Linear,
            mag_filter: SamplerFilter::Linear,
            mipmap_filter: SamplerFilter::Linear,
            address_mode: SamplerAddressMode::Repeat
        });
        (image_type, texture, sampler)
    };
//text
    let font = Font::new(include_bytes!("../res/LatiniaBlack.ttf"));
    let chars = Font::digits();
    let (atlas_data, atlas) = Atlas::new(font, 300.0, chars, ATLAS_SIZE, 3);
    let atlas_image_type = ImageType { channel: ImageChannelType::RUnorm, width: ATLAS_SIZE, height: ATLAS_SIZE, layers: ImageLayers::Array(atlas_data.len() as u32) };
    let atlas_image = device.new_image(atlas_image_type, ImageUsage::Texture { mipmapping: false });
    {
        let mut buffer = device.new_image_buffer(atlas_image_type);
        let mut fence = device.new_fence(false);
        for i in 0..atlas_data.len()
        {
            buffer.write(&atlas_data[i]);
            let image_fence = command_pool.new_command_buffer().copy_to_image(&graphic_queue.lock().unwrap(), &buffer, &atlas_image, i as u32, fence);
            image_fence.mark.wait();
            fence = image_fence.mark;
            fence.reset();
        }
    }
    let mut text_vertices = Vec::with_capacity(4 * consts::SCORE_DIGITS);
    let mut text_indices = Vec::with_capacity(6 * consts::SCORE_DIGITS);
//dynamic buffer
    let mut buffer_layout = device.new_buffer_type();
    let cam_view = buffer_layout.add_uniforms(1);
    let light_view = buffer_layout.add_uniforms(1);
    let text_uniform_view = buffer_layout.add_uniforms(1);
    let text_vertex_view = buffer_layout.add_attributes(4 * consts::SCORE_DIGITS as u32);
    let text_index_view = buffer_layout.add_indices(6 * consts::SCORE_DIGITS as u32);
//gerenerate and fill flash data
    let (mut dynamic_buffers, flash_vertex_view, flash_index_view, flash_instance_view) =
    {
        let (vertices, indices) = marching_cubes::mesh_builder
        (
            Vec3(0.0, 0.0, 0.0),
            Vec3(consts::FLASH_HEIGHT + consts::FLASH_EPS, consts::FLASH_RADIUS + consts::FLASH_EPS, consts::FLASH_RADIUS + consts::FLASH_EPS),
            (consts::FLASH_RESOLUTION, consts::FLASH_RESOLUTION, consts::FLASH_RESOLUTION),
            &flash::FlashMold()
        );
        let vertices: Vec<_> = vertices.iter().map(|vertex| flash::FlashVertex { pos: vertex.position.into() }).collect();
        let vertex_view = buffer_layout.add_attributes(vertices.len() as u32);
        let index_view = buffer_layout.add_indices(indices.len() as u32);
        let instance_view = buffer_layout.add_attributes((consts::BLOCK_SPAWN_FRONT_DISTANCE + consts::BLOCK_DESPAWN_BACK_DISTANCE) as u32 * 2);
        let buffer_layout = buffer_layout.build();
        let mut buffers = swapchain.new_objects(&mut |_| device.new_buffer(&buffer_layout, BufferUsage::Dynamic));
        for buffer in buffers.iter_mut()
        {
            let mut map = buffer.map();
            map.write_attributes(&vertex_view, 0, &vertices);
            map.write_indices(&index_view, 0, &indices);
        }
        (buffers, vertex_view, index_view, instance_view)
    };
    let mut flashes = Vec::new();
//descriptors
    let cam_descriptor_layout = device.new_descriptor_set_layout(0, vec![DescriptorBindingInfo::from_struct::<CamBinding>(1, true, false)]);
    let light_descriptor_layout = device.new_descriptor_set_layout(1, vec![DescriptorBindingInfo::from_struct::<LightBinding>(1, true, true)]);
    let text_descriptor_layout = device.new_descriptor_set_layout(0, vec![DescriptorBindingInfo::from_struct::<TextBinding>(1, true, true), DescriptorBindingInfo::from_sampler(atlas_image_type.channel, 1, false, true)]);
    let mut uniform_descriptors = swapchain.new_objects(&mut |_| device.new_descriptor_sets(&[(&cam_descriptor_layout, 1), (&light_descriptor_layout, 1), (&text_descriptor_layout, 1)]));
    for (descriptor, buffer) in uniform_descriptors.iter_mut().zip(dynamic_buffers.iter())
    {
        descriptor[0][0].update_struct(0, &buffer, &cam_view);
        descriptor[1][0].update_struct(0, &buffer, &light_view);
        descriptor[2][0].update_struct(0, &buffer, &text_uniform_view);
        descriptor[2][0].update_sampler(1, &[&atlas_image], &sampler);
    }
    let tex_descriptor_layout = device.new_descriptor_set_layout(2, vec![DescriptorBindingInfo::from_sampler(image_type.channel, 1, false, true)]);
    let mut tex_descriptor = device.new_descriptor_sets(&[(&tex_descriptor_layout, 1)]).remove(0).remove(0);
    tex_descriptor.update_sampler(0, &[&texture], &sampler);
//cave
    let seed = (std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)).unwrap().as_nanos() as u32;
    let mold_gen = ||
    {
        let mut billow = noise::Billow::new().set_seed(seed);
        billow.octaves = consts::CAVE_GEN_OCTAVES;
        billow.frequency = consts::CAVE_GEN_FREQUENCY;
        billow.lacunarity = consts::CAVE_GEN_LUCUNARITY;
        billow.persistence = consts::CAVE_GEN_PERSISTANCE;
        cave::Cave::new(billow, noise::Perlin::new().set_seed(seed), consts::CAVE_GEN_BIAS)
    };
    let mold = mold_gen();
    let light_perlin = noise::Perlin::new();
	let blocks = std::cell::RefCell::new(HashMap::<i32, cave::CylinderBlock>::new());
    let mut blocks_requested = HashSet::new();
    let mut block_graveyard = swapchain.new_cycle(&mut || Vec::new());
    let generators = vec!
    [
        cave::BlockGenerator::new(&device, graphic_queue_family_info, mold_gen()),
        cave::BlockGenerator::new(&device, graphic_queue_family_info, mold_gen()),
        cave::BlockGenerator::new(&device, graphic_queue_family_info, mold_gen())
    ];
    let mut generator_index = 0;
//cam
    let mut cam = camera::Camera::new();
    cam.build_projection(width as f32 / height as f32);
    cam.pos.0 = mold.x0();
    cam.pos.1 = mold.y0();
//main graphic stuff
    let msaa = Msaa::X4;
    //image buffers
    let color_buffer = device.new_image(ImageType { channel: Swapchain::IMAGE_CHANNEL_TYPE, width, height, layers: ImageLayers::Single }, ImageUsage::Attachment { depth: false, samples: msaa, texture: false, transfer_src: false });
    let depth_buffer = device.new_image(ImageType { channel: ImageChannelType::DSfloat, width, height, layers: ImageLayers::Single }, ImageUsage::Attachment { depth: true, samples: msaa, texture: false, transfer_src: false });
    //renderpass & pipeline creation
    let render_pass = device.new_render_pass
    (
        &[
            RenderPassColorAttachment::Image
            {
                image_channel_type: Swapchain::IMAGE_CHANNEL_TYPE,
                samples: msaa,
                load: ColorAttachmentLoad::DontCare,
                store: AttachmentStore::Store,
                initial_layout: ImageLayout::Undefined,
                final_layout: ImageLayout::Attachment
            },
            RenderPassColorAttachment::Swapchain(SwapchainLoad::DontCare)
        ],
        Some(RenderPassDepthAttachment
        {
            image_channel_type: ImageChannelType::DSfloat,
            samples: msaa,
            load: DepthAttachmentLoad::Clear { depth: 1.0 },
            store: AttachmentStore::DontCare,
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::Attachment
        }),
        &[Subpass
        {
            input_attachments: &[],
            output_attachments: &[OutputAttachment { attachment_index: 0, fragment_out_location: 0 }],
            resolve_attachments: Some(&[ResolveAttachment::Index(1)]),
            depth_attachment: true
        }]
    );
    let framebuffers = swapchain.new_objects(&mut |index| device.new_framebuffer(&render_pass, &[FramebufferAttachment::Image(&color_buffer), FramebufferAttachment::Swapchain(swapchain.get_image(index)), FramebufferAttachment::Image(&depth_buffer)]));
    let pipeline_layout = device.new_pipeline_layout(&[&cam_descriptor_layout, &light_descriptor_layout, &tex_descriptor_layout], None);
    let text_pipeline_layout = device.new_pipeline_layout(&[&text_descriptor_layout], None);
    let mut pipeline_info = PipelineInfo
    {
        view: Some(ViewInfo::full(width, height)),
        topology: PipelineTopology::TriangleList,
        samples: msaa,
        min_sample_shading: None,
        line_width: 1.0,
        polygon: PipelinePolygon::Fill,
        cull: PipelineCull::Back,
        depth_test: true,
        blend: false
    };
    //pipeline_info.cull = PipelineCull::None;
    let cave_pipeline = device.new_pipeline
    (
        &render_pass, 0,
        CAVE_VERTEX, CAVE_FRAGMENT,
        &[AttributeGroupInfo::from::<Vertex>()], &pipeline_layout,
        &pipeline_info
    );
    let flash_pipeline = device.new_pipeline
    (
        &render_pass, 0,
        FLASH_VERTEX, FLASH_FRAGMENT,
        &[AttributeGroupInfo::from::<flash::FlashVertex>(), AttributeGroupInfo::from::<flash::FlashInstance>()], &pipeline_layout,
        &pipeline_info
    );
    let bg_pipeline = device.new_pipeline
    (
        &render_pass, 0,
        BG_VERTEX, BG_FRAGMENT,
        &[], &pipeline_layout,
        &pipeline_info
    );
    pipeline_info.depth_test = false;
    pipeline_info.blend = true;
    let text_pipeline = device.new_pipeline
    (
        &render_pass, 0,
        TEXT_VERTEX, TEXT_FRAGMENT,
        &[AttributeGroupInfo::from::<TextVertex>()], &text_pipeline_layout,
        &pipeline_info
    );
    //synchronization elements
    let image_available = swapchain.new_cycle(&mut || device.new_semaphore());
    let rendering_finished = swapchain.new_cycle(&mut || device.new_semaphore());
    let may_begin_drawing = swapchain.new_cycle(&mut || device.new_fence(true));
//command buffer creation and filling
    let command_buffers = std::cell::RefCell::new(swapchain.new_objects(&mut |_| command_pool.new_command_buffer()));
    let commander = |image_index: &SwapchainObjectIndex, dynamic_buffer: &Buffer, flash_count: u32, text_index_count: u32|
    {
        let mut command_buffers_borrow = command_buffers.borrow_mut();
        let command_buffer = command_buffers_borrow.get_mut(image_index);
        let framebuffer = framebuffers.get(image_index);
        let uniform_descriptor = uniform_descriptors.get(image_index);
        let mut record = command_buffer.record();
        let mut pass = record.render_pass(&render_pass, &framebuffer);
        pass
            .bind_descriptor_sets(&pipeline_layout, &[&uniform_descriptor[0][0], &uniform_descriptor[1][0], &tex_descriptor])
            .bind_pipeline(&cave_pipeline);
        for block in blocks.borrow().values()
        {
            pass
                .bind_attributes(0, [AttributeBinding::from::<Vertex>(&block.buffer, &block.vertex_view)])
                .bind_indices(IndexBinding::from(&block.buffer, &block.index_view))
                .draw(DrawMode::index(block.index_view.count()));
        }
        pass
            .bind_pipeline(&flash_pipeline)
            .bind_attributes(0, [
                AttributeBinding::from::<flash::FlashVertex>(&dynamic_buffer, &flash_vertex_view),
                AttributeBinding::from::<flash::FlashInstance>(&dynamic_buffer, &flash_instance_view)
            ])
            .bind_indices(IndexBinding::from(&dynamic_buffer, &flash_index_view))
            .draw(DrawMode::index_instanced(flash_index_view.count(), flash_count));
        pass
            .bind_pipeline(&bg_pipeline)
            .draw(DrawMode::vertex(36));
        pass
            .bind_pipeline(&text_pipeline)
            .bind_descriptor_sets(&text_pipeline_layout, &[&uniform_descriptor[2][0]])
            .bind_attributes(0, [AttributeBinding::from::<TextVertex>(&dynamic_buffer, &text_vertex_view)])
            .bind_indices(IndexBinding::from(&dynamic_buffer, &text_index_view))
            .draw(DrawMode::index(text_index_count));
    };
    let mut rerecord = swapchain.new_objects(&mut |_| true);
//screenshots
    let mut shot = false;
    let mut shot_command_buffer: Option<CommandBuffer> = None;
//game loop
    let mut fps = FPS::new(None);
    let mut time = -consts::WAIT_TIME;
    let mut ambient_flash = Vec3(0.0, 0.0, 0.0);
    window.set_visible(true);
    use winit::platform::run_return::EventLoopExtRunReturn;
    event_loop.run_return(|event, _, control_flow|
    {
        match event
        {
            event::Event::WindowEvent { window_id: _, event } =>
            {
                match event
                {
                    event::WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    event::WindowEvent::KeyboardInput { device_id: _, input, is_synthetic: _ } =>
                    {
                        if let Some(virtual_keycode) = input.virtual_keycode
                        {
                            match virtual_keycode
                            {
                                VirtualKeyCode::W => cam.forward = input.state == ElementState::Pressed,
                                VirtualKeyCode::S => cam.backward = input.state == ElementState::Pressed,
                                VirtualKeyCode::A => cam.left = input.state == ElementState::Pressed,
                                VirtualKeyCode::D => cam.right = input.state == ElementState::Pressed,
                                VirtualKeyCode::Space => cam.jump(),
                                VirtualKeyCode::K => cam.does_physics = false,
                                VirtualKeyCode::L => cam.does_physics = true,
                                VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                                VirtualKeyCode::P => if input.state == ElementState::Pressed { shot = true },
                                _ => {}
                            };
                        }
                    },
                    _ => {}
                };
            },
            winit::event::Event::DeviceEvent { device_id: _, event } =>
            {
                match event
                {
                    winit::event::DeviceEvent::MouseMotion { delta } =>
                    {
                        cam.phi += consts::MOUSE_SENSITIVITY * delta.0 as f32;
                        cam.theta -= consts::MOUSE_SENSITIVITY * delta.1 as f32;
                        cam.theta = cam.theta.max(-std::f32::consts::FRAC_PI_2).min(std::f32::consts::FRAC_PI_2);
                    },
                    _ => {}
                }
            },
            winit::event::Event::MainEventsCleared =>
            {
                //logic
                let dt = fps.dt();
                time += dt;
                ambient_flash = ambient_flash * consts::FLASH_AMBIENT_DECAY.powf(dt);
                cam.logic(dt, &mold);
                block_graveyard.get_mut(&swapchain).clear();
                let mut blocks_changed = false;

                let cam_norm = cam.pos.2 / consts::BLOCK_LENGTH;
                for block_z in &[cam_norm.floor() as i32, cam_norm.ceil() as i32]
                {
                    if let hash_map::Entry::Occupied(mut entry) = blocks.borrow_mut().entry(*block_z)
                    {
                        let block = entry.get_mut();
                        for i in 0..block.flashes.len()
                        {
                            if (cam.pos + block.flashes[i].0 * (-1.0)).norm() < consts::PICKUP_RANGE
                            {
                                time -= consts::FLASH_POWER;
                                ambient_flash = ambient_flash + block.flashes[i].1 * consts::FLASH_AMBIENT_POWER;
                                block.flashes.remove(i);
                                blocks_changed = true;
                                break;
                            }
                        }
                    }
                }
                //fetch generated blocks
                for generator in &generators
                {
                    for block in generator.receive()
                    {
                        blocks_changed = true;
                        blocks_requested.remove(&block.z);
                        if let Some(block) = blocks.borrow_mut().insert(block.z, block)
                        {
                            println!("Regenerated block {}!", block.z);
                            block_graveyard.get_mut(&swapchain).push(block);
                        }
                    }
                }
                //check blocks needed
                let range = (cam_norm.round() as i32)..=(cam_norm.round() as i32 + consts::BLOCK_SPAWN_FRONT_DISTANCE);
                for block_needed in range
                {
                    if blocks.borrow().values().find(|block| block.z == block_needed).is_none() && blocks_requested.iter().find(|block| **block == block_needed).is_none()
                    {
                        generators[generator_index].request(block_needed);
                        generator_index = (generator_index + 1) % generators.len();
                        blocks_requested.insert(block_needed);
                    }
                }
                //check blocks unneeded
                let mut blocks_remove = Vec::new();
                for block in blocks.borrow().values()
                {
                    if (cam_norm - block.z as f32).floor() as i32 >= consts::BLOCK_DESPAWN_BACK_DISTANCE { blocks_remove.push(block.z); }
                }
                if !blocks_remove.is_empty() { blocks_changed = true; }
                for block in blocks_remove { block_graveyard.get_mut(&swapchain).push(blocks.borrow_mut().remove(&block).unwrap()); }
                if blocks_changed { rerecord.iter_mut().for_each(|b| *b = true); }
                if blocks_changed
                {
                    flashes.clear();
                    for block in blocks.borrow().values()
                    {
                        for flash in block.flashes.iter()
                        {
                            flashes.push(flash::FlashInstance { offset: flash.0.into(), color: flash.1.into() });
                        }
                    }
                }
                //compute score
                let z_bias = consts::Z_BIAS_OFFSET + consts::C * time.max(0.0);
                let score = consts::MAX_BIAS - (z_bias - cam.pos.2);
                if score <= 0.0 { *control_flow = ControlFlow::Exit; }
                let score_digits = format!("{:03}", score.round() as u32);
                text_vertices.clear();
                text_indices.clear();
                atlas.text
                (
                    &score_digits,
                    Layout { width: std::f32::MAX, align: Align::Left, auto_wrap: false },
                    &mut |i| text_indices.push(i as u16),
                    &mut |(cx, cy, cl), p: (f32, f32)| text_vertices.push(TextVertex { position: p.into(), tex_coords: (cx, cy, cl as f32).into() })
                );
                //render
                let image_available = image_available.get(&swapchain);
                let rendering_finished = rendering_finished.get(&swapchain);
                let may_begin_drawing = may_begin_drawing.get(&swapchain);
                let maybe_image_index = swapchain.acquire_next_image(Some(&image_available), None);
                if let Ok(image_index) = maybe_image_index
                {
                    may_begin_drawing.wait();
                    may_begin_drawing.reset();
                    {
                        let light_on = light_perlin.get([time as f64 * consts::LIGHT_FREQUENCY, 0.0]) + consts::LIGHT_BIAS > 0.0;
                        let (proj, trans) = cam.mats();
                        let dir = trans.transpose() * Mat4::rotation_x(consts::LIGHT_ANGLE) * Vec4(0.0, 0.0, 1.0, 0.0);
                        let dir = (dir.0, dir.1, dir.2);
                        let mut map = dynamic_buffers.get_mut(&image_index).map();
                        map.write_uniforms(&cam_view, 0, &[CamBinding { mat: proj * trans }]);
                        map.write_uniforms(&light_view, 0, &[LightBinding
                        {
                            z_bias,
                            color: if light_on { consts::LIGHT_COLOR } else { (0.0, 0.0, 0.0) },
                            pos: (cam.pos + Vec3(0.0, -consts::FIGUR_HEIGHT, 0.0)).into(),
                            dir,
                            cos_angle_inner: consts::LIGHT_ANGLE_INNER.cos(),
                            cos_angle_outer: consts::LIGHT_ANGLE_OUTER.cos(),
                            ambient: consts::AMBIENT_LIGHT_COLOR,
                            flash_ambient: ambient_flash.into(),
                            _padding0: Default::default(),
                            _padding1: Default::default(),
                            _padding2: Default::default(),
                            _padding3: Default::default(),
                            _padding4: Default::default()
                        }]);
                        map.write_attributes(&flash_instance_view, 0, &flashes);
                        map.write_attributes(&text_vertex_view, 0, &text_vertices);
                        map.write_indices(&text_index_view, 0, &text_indices);
                        map.write_uniforms(&text_uniform_view, 0, &[TextBinding { aspect: width as f32 / height as f32, height: 0.1 }]);
                    }
                    let rerecord = rerecord.get_mut(&image_index);
                    if *rerecord
                    {
                        commander(&image_index, &dynamic_buffers.get(&image_index), flashes.len() as u32, text_indices.len() as u32);
                        *rerecord = false;
                    }
                    let graphic_queue = graphic_queue.lock().unwrap();
                    command_buffers.borrow().get(&image_index).submit(&graphic_queue, Some(&image_available), Some(&rendering_finished), &may_begin_drawing);
                    if shot
                    {
                        let image_type = ImageType { channel: Swapchain::IMAGE_CHANNEL_TYPE, width, height, layers: ImageLayers::Single };
                        let buffer = device.new_image_buffer(image_type);
                        let fence = command_pool.new_command_buffer().copy_from_image(&graphic_queue, CopyImageSource::Swapchain(swapchain.get_image(&image_index)), &buffer, device.new_fence(false));
                        shot_command_buffer = Some(fence.command_buffer);
                        let fence = fence.mark;
                        std::thread::spawn(move ||
                        {
                            fence.wait();
                            let mut image = Vec::with_capacity(buffer.size());
                            image.resize(buffer.size(), 0);
                            buffer.read(&mut image);
                            let image = image::DynamicImage::ImageBgra8(image::ImageBuffer::from_raw(width, height, image).unwrap()).into_rgba8();
                            image.save_with_format("screenshot.png", image::ImageFormat::Png).unwrap();
                            println!("screenshot saved");
                        });
                        shot = false;
                    }
                    swapchain.present(image_index, &graphic_queue, &rendering_finished);
                }
            },
            _ => {}
        }
    });
//wait for shutdown
    for generator in generators { generator.shutdown(); }
    device.idle();
}
