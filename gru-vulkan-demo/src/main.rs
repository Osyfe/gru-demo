//no console
#![windows_subsystem = "windows"]

use gru_vulkan::*;
use gru_misc::{math::*, text::*, time::*};
use winit::{*, event_loop::ControlFlow, event::{VirtualKeyCode, MouseButton, ElementState}};

//compile glsl to spirv statically!
const VERTEX: Shader = vert_shader!("src/glsl/shader.vert");
const FRAGMENT: Shader = frag_shader!("src/glsl/shader.frag");

//configs
const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;

const ATLAS_SIZE: u32 = 1024;
const TEXT_WIDTH: f32 = 20.0;

const MSAA: Msaa = Msaa::X4; //cannot be None here (due to the different renderpass setup)
const V_SYNC: bool = true;
const FPS_CAP: Option<usize> = None;

fn main()
{
    //window
    let mut event_loop = event_loop::EventLoop::new();
    let window = window::WindowBuilder::new()
        .with_title("gru-vulkan demo")
        .with_inner_size(dpi::PhysicalSize { width: WIDTH as f32, height: HEIGHT as f32 })
        .with_visible(false)
        .with_resizable(false) //would requiere a swapchain recreation which we don't do here
        .build(&event_loop).unwrap();
    //vulkan initialization
    let instance = Instance::new(Some(&window));
    let physical_devices = instance.physical_devices();
    let gpu = &physical_devices[0]; //pick first gpu
    let queue_family_info = &gpu.queue_families()[0]; //expects an allrounder queue family here (at least graphic + transfer)
    let device = instance.logical_device(gpu, &[(queue_family_info, &[1.0])]);
    //queue fetching
    let queue_family = device.get_queue_family(queue_family_info);
    let queue_arc = queue_family.get_queue(0);
    let queue = queue_arc.lock().unwrap();
    let command_pool = device.new_command_pool(queue_family);
    //swapchain
    let swapchain = device.new_swapchain(None, V_SYNC).unwrap();
    let (width, height) = swapchain.dimensions();
    //font
    let (atlas, atlas_image) =
    {
        let font = Font::new(include_bytes!("res/futuram.ttf"));
        let chars = &Font::all_letters() | &Font::text_special_characters();
        let (atlas_data, atlas) = Atlas::new(&font, &chars, 200.0, ATLAS_SIZE, 5);
        let atlas_image_type = ImageType { channel: ImageChannelType::RUnorm, width: ATLAS_SIZE, height: ATLAS_SIZE, layers: Some(atlas_data.len() as u32) };
        let atlas_image = device.new_image(atlas_image_type, ImageUsage::Texture { mipmapping: true });
        let mut atlas_buffer = device.new_image_buffer(atlas_image_type);
        let mut fence = device.new_fence(false);
        for i in 0..atlas_data.len()
        {
            atlas_buffer.write(&atlas_data[i]);
            let copy_fence = command_pool.new_command_buffer().copy_to_image(&queue, &atlas_buffer, &atlas_image, i as u32, fence);
            copy_fence.mark.wait();
            fence = copy_fence.mark;
        }
        (atlas, atlas_image)
    };
    //text data
    let mut indices = Vec::new();
    let mut vertices = Vec::new();
    let TextData { index_count, line_count, .. } = atlas.text
    (
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.",
        TEXT_WIDTH,
        Align::Block,
        &mut |i| indices.push(i),
        &mut |c, p| vertices.push(Char { position: p.into(), coords: c.into() })
    );
    //text data upload
    let mut buffer_type = device.new_buffer_type();
    let index_view = buffer_type.add_indices(indices.len());
    let vertex_view = buffer_type.add_attributes(vertices.len());
    let text_buffer =
    {
        let mut stage_buffer = device.new_buffer(&mut buffer_type, BufferUsage::Stage);
        {
            let mut map = stage_buffer.map();
            map.write_indices(&index_view, 0, &indices);
            map.write_attributes(&vertex_view, 0, &vertices);
        }
        let text_buffer = device.new_buffer(&mut buffer_type, BufferUsage::Static);
        command_pool.new_command_buffer().copy_buffer(&queue, &stage_buffer, &text_buffer, device.new_fence(false)).mark.wait();
        text_buffer
    };
    //camera
    let mut cam = Camera::new(width as f32 / height as f32, line_count as f32);
    let mut buffer_type = device.new_buffer_type();
    let cam_view = buffer_type.add_uniforms(1);
    let mut cam_buffers = swapchain.new_objects(&mut |_| device.new_buffer(&mut buffer_type, BufferUsage::Dynamic));
    //sampler
    let sampler = device.new_sampler(&SamplerInfo
    {
        min_filter: SamplerFilter::Linear,
        mag_filter: SamplerFilter::Linear,
        mipmap_filter: SamplerFilter::Linear,
        address_mode: SamplerAddressMode::ClampToEdge
    });
    //descriptor
    let descriptor_layout = device.new_descriptor_set_layout(0, vec!
    [
        DescriptorBindingInfo::from_struct::<CamBinding>(1, true, false), //binding 0
        DescriptorBindingInfo::from_sampler(ImageChannelType::RUnorm, 1, false, true) //binding 1
    ]);
    let mut descriptors = swapchain.new_objects(&mut |_| device.new_descriptor_sets(&[(&descriptor_layout, 1)]).remove(0).remove(0));
    for (descriptor, cam_buffer) in descriptors.iter_mut().zip(cam_buffers.iter())
    {
        descriptor.update_struct(0, &cam_buffer, &cam_view);
        descriptor.update_sampler(1, &[&atlas_image], &sampler);
    }
    //image buffers
    let color_buffer = device.new_image(ImageType { channel: Swapchain::IMAGE_CHANNEL_TYPE, width, height, layers: None }, ImageUsage::Attachment { depth: false, samples: MSAA, texture: false, transfer_src: false });
    let depth_buffer = device.new_image(ImageType { channel: ImageChannelType::DSfloat, width, height, layers: None }, ImageUsage::Attachment { depth: true, samples: MSAA, texture: false, transfer_src: false });
    //renderpass (assumes some level of MSAA)
    let render_pass = device.new_render_pass
    (
        &[&RenderPassColorAttachment::Image
        {
            image_channel_type: Swapchain::IMAGE_CHANNEL_TYPE,
            samples: MSAA,
            load: ColorAttachmentLoad::Clear { color: [0.02, 0.05, 0.0, 1.0] },
            store: AttachmentStore::Store,
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::Attachment
        },
        &RenderPassColorAttachment::Swapchain(SwapchainLoad::DontCare)],
        Some(&RenderPassDepthAttachment
        {
            image_channel_type: ImageChannelType::DSfloat,
            samples: MSAA,
            load: DepthAttachmentLoad::Clear { depth: 1.0 },
            store: AttachmentStore::DontCare,
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::Attachment
        }),
        &[&Subpass
        {
            input_attachments: &[],
            output_attachments: &[OutputAttachment { attachment_index: 0, fragment_out_location: 0 }],
            resolve_attachments: Some(&[ResolveAttachment::Index(1)]),
            depth_attachment: true
        }]
    );
    let framebuffers = swapchain.new_objects(&mut |index| device.new_framebuffer(&render_pass, &[&FramebufferAttachment::Image(&color_buffer), &FramebufferAttachment::Swapchain(swapchain.get_image(index)), &FramebufferAttachment::Image(&depth_buffer)]));
    //pipeline
    let pipeline_layout = device.new_pipeline_layout(&[&descriptor_layout]);
    let pipeline_info = PipelineInfo
    {
        viewport_origin: (0.0, 0.0),
        viewport_size: (width as f32, height as f32),
        scissor_origin: (0, 0),
        scissor_size: (width, height),
        topology: PipelineTopology::TriangleList,
        samples: MSAA,
        min_sample_shading: None,
        line_width: 1.0,
        polygon: PipelinePolygon::Fill,
        cull: PipelineCull::Back,
        depth_test: true,
        blend: true
    };
    let pipeline = device.new_pipeline(&render_pass, 0, VERTEX, FRAGMENT, &[&AttributeGroupInfo::from::<Char>()], &pipeline_layout, &pipeline_info);
    //command buffers (we reuse them throughout the program)
    let mut command_buffers = swapchain.new_objects(&mut |_| command_pool.new_command_buffer());
    for ((command_buffer, framebuffer), descriptor) in command_buffers.iter_mut().zip(framebuffers.iter()).zip(descriptors.iter())
    {
        command_buffer
            .record()
            .render_pass(&render_pass, &framebuffer)
            .bind_descriptor_sets(&pipeline_layout, &[&descriptor])
            .bind_pipeline(&pipeline)
            .bind_attributes([&AttributeBinding::from(&text_buffer, &vertex_view)])
            .draw(&DrawMode::Index(IndexBinding::from(&text_buffer, &index_view), index_count), 1);
    }
    //synchronization elements
    let image_available = swapchain.new_cycle(&mut || device.new_semaphore());
    let rendering_finished = swapchain.new_cycle(&mut || device.new_semaphore());
    let may_begin_drawing = swapchain.new_cycle(&mut || device.new_fence(true));
    //game loop
    let mut fps = FPS::new(FPS_CAP);
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
                                VirtualKeyCode::Space => cam.up = input.state == ElementState::Pressed,
                                VirtualKeyCode::LShift => cam.down = input.state == ElementState::Pressed,
                                VirtualKeyCode::F11 => if input.state == ElementState::Pressed
                                {
                                    match window.fullscreen()
                                    {
                                        None =>  window.set_fullscreen(Some(window::Fullscreen::Borderless(None))),
                                        Some(_) => window.set_fullscreen(None)
                                    };
                                },
                                VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                                _ => {}
                            };
                        }
                    },
                    event::WindowEvent::MouseInput { state, button, .. } =>
                    {
                        if button == MouseButton::Right { cam.rotating = state == ElementState::Pressed }
                    }
                    _ => {}
                };
            },
            event::Event::DeviceEvent { device_id: _, event } =>
            {
                match event
                {
                    event::DeviceEvent::MouseMotion { delta } => if cam.rotating
                    {
                        cam.phi += 0.002 * delta.0 as f32;
                        cam.theta -= 0.002 * delta.1 as f32;
                        cam.theta = cam.theta.max(-std::f32::consts::FRAC_PI_2).min(std::f32::consts::FRAC_PI_2);
                    },
                    _ => {}
                }
            },
            event::Event::MainEventsCleared =>
            {
                //logic
                let dt = fps.dt();
                cam.walk(dt);
                //render
                let image_available = image_available.get(&swapchain);
                let rendering_finished = rendering_finished.get(&swapchain);
                let may_begin_drawing = may_begin_drawing.get(&swapchain);
                let maybe_image_index = swapchain.acquire_next_image(Some(&image_available), None);
                if let Ok(image_index) = maybe_image_index
                {
                    //wait for availability
                    may_begin_drawing.wait();
                    may_begin_drawing.reset();
                    //update dynamic cam buffer
                    {
                        let mut map = cam_buffers.get_mut(&image_index).map();
                        map.write_uniforms(&cam_view, 0, &[CamBinding { mat: cam.mat().into() }]);
                    }
                    //submit and presend
                    command_buffers.get(&image_index).submit(&queue, &image_available, &rendering_finished, &may_begin_drawing);
                    swapchain.present(image_index, &queue, &rendering_finished);
                } else
                { //some swapchain error occured (such as a resize or the window was minimized)
                    println!("Swapchain error!");
                    *control_flow = ControlFlow::Exit;
                }
            },
            _ => {}
        }
    });
    //wait for task completion before dropping all vulkan resources
    device.idle();
}

//shader vertex input
#[derive(VertexAttributeGroupReprCpacked)]
#[repr(C, packed)]
struct Char
{
    #[location = 0]
    position: F2,
    #[location = 1]
    coords: F3
}

//shader uniform input (watch out for std140 layout!)
#[derive(Clone, Copy, DescriptorStructReprC)]
#[repr(C)]
struct CamBinding
{
    mat: Mat4
}

//holds all camera variables
struct Camera
{
    proj: Mat4,
    theta: f32,
    phi: f32,
    pos: (f32, f32, f32),
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    rotating: bool
}

impl Camera
{
    //init the camera
    fn new(aspect: f32, text_height: f32) -> Self
    {
        Camera
        {
            proj: Mat4::perspective_vulkan(aspect, std::f32::consts::FRAC_PI_8, 0.1, 100.0),
            theta: 0.0,
            phi: 0.0,
            pos: (TEXT_WIDTH / 2.0, text_height / 2.0, -30.0),
            forward: false,
            backward: false,
            left: false,
            right: false,
            up: false,
            down: false,
            rotating: false
        }
    }
    //logic
    fn walk(&mut self, dt: f32)
    {
        fn front(cam: &Camera) -> (f32, f32, f32) { (cam.theta.cos() * cam.phi.sin(), -cam.theta.sin(), cam.theta.cos() * cam.phi.cos()) }
        fn left(cam: &Camera) -> (f32, f32, f32) { (-cam.phi.cos(), 0.0, cam.phi.sin()) }
        fn up(_cam: &Camera) -> (f32, f32, f32) { (0.0, -1.0, 0.0) }
        fn walk(cam: &mut Camera, dir: (f32, f32, f32), sign: f32, dt: f32)
        {
            let speed = 10.0 * sign * dt;
            cam.pos.0 += speed * dir.0;
            cam.pos.1 += speed * dir.1;
            cam.pos.2 += speed * dir.2;
        }
        if self.forward { walk(self, front(self), 1.0, dt); }
        if self.backward { walk(self, front(self), -1.0, dt); }
        if self.left { walk(self, left(self), 1.0, dt); }
        if self.right { walk(self, left(self), -1.0, dt); }
        if self.up { walk(self, up(self), 1.0, dt); }
        if self.down { walk(self, up(self), -1.0, dt); }
    }
    //the total world space -> clip space transform
    fn mat(&self) -> Mat4
    {
        let rot = Mat4::rotation_x(-self.theta) * Mat4::rotation_y(-self.phi);
        let trans = Mat4::translation(-self.pos.0, -self.pos.1, -self.pos.2);
        self.proj * rot * trans
    }
}
