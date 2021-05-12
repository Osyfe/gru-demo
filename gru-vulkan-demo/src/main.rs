//no console
//#![windows_subsystem = "windows"]

use gru_vulkan::*;
use gru_misc::
{
    math::*,
    text::*,
    time::*
};
use winit::
{
    *,
    event_loop::ControlFlow,
    event::
    {
        VirtualKeyCode,
        MouseButton,
        ElementState
    }
};

const ATLAS_SIZE: u32 = 1024;

const VERTEX: Shader = vert_shader!("src/glsl/shader.vert");
const FRAGMENT: Shader = frag_shader!("src/glsl/shader.frag");

#[derive(VertexAttributeGroupReprCpacked)]
#[repr(C, packed)]
pub struct Char
{
    #[location = 0]
    pub position: F2,
    #[location = 1]
    pub coords: F3
}

#[derive(Clone, Copy, DescriptorStructReprC)]
#[repr(C)]
pub struct CamBinding
{
    mat: Mat4
}

fn main()
{
    //window setup
    let mut event_loop = event_loop::EventLoop::new();
    let window = window::WindowBuilder::new()
        .with_title("gru-vulkan demo")
        .with_inner_size(dpi::PhysicalSize { width: 512.0, height: 512.0 })
        .with_visible(false)
        .with_resizable(true)
        .build(&event_loop).unwrap();
    //initialization and queue fetching
    let instance = Instance::new(Some(&window));
    let physical_devices = instance.physical_devices();
    let geforce = &physical_devices[0];
    let graphic_queue_family_info = &geforce.queue_families()[0];
    //let transfer_queue_family_info = &geforce.queue_families()[1];
    let device = instance.logical_device(geforce, vec![(graphic_queue_family_info, vec![1.0])]);//, (transfer_queue_family_info, vec![1.0])]);
    let graphic_queue_family = device.get_queue_family(graphic_queue_family_info);
    let graphic_queue_arc = graphic_queue_family.get_queue(0);
    let graphic_queue = graphic_queue_arc.lock().unwrap();
    //let transfer_queue_family = device.get_queue_family(transfer_queue_family_info);
    //let transfer_queue = transfer_queue_family.get_queue(0);
    let graphic_command_pool = device.new_command_pool(graphic_queue_family);
    //let transfer_command_pool = device.new_command_pool(transfer_queue_family);
    //font setup
    let font = Font::new(include_bytes!("res/futuram.ttf"));
    let chars = &Font::all_letters() | &Font::text_special_characters();
    let (atlas_data, atlas) = Atlas::new(&font, &chars, 200.0, ATLAS_SIZE, 5);
    /*
    for i in 0..atlas_data.len()
    {
        let bitmap: image::GrayImage = image::ImageBuffer::from_raw(atlas_image_type.width, atlas_image_type.height, atlas_data[i].clone()).unwrap();
        bitmap.save_with_format(format!("bitmap{}.png", i), image::ImageFormat::Png).ok();
    }
    */
    let atlas_image_type = ImageType { channel: ImageChannelType::RUnorm, width: ATLAS_SIZE, height: ATLAS_SIZE, layers: Some(atlas_data.len() as u32) };
    let atlas_image = device.new_image(atlas_image_type, ImageUsage::Texture { mipmapping: true });
    let mut atlas_buffer = device.new_image_buffer(atlas_image_type);
    let mut fence = device.new_fence(false);
    for i in 0..atlas_data.len()
    {
        atlas_buffer.write(&atlas_data[i]);
        let copy_fence = graphic_command_pool.new_command_buffer().copy_to_image(&graphic_queue, &atlas_buffer, &atlas_image, i as u32, fence);
        copy_fence.mark.wait();
        fence = copy_fence.mark;
    }
    let mut indices = Vec::new();
    let mut vertices = Vec::new();
    let TextData { index_count, .. } = atlas.text
    (
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.",
        20.0,
        Align::Block,
        &mut |i| indices.push(i),
        &mut |c, p| vertices.push(Char { position: p.into(), coords: c.into() })
    );
    //camera setup
    let mut buffer_layout = device.new_buffer_layout();
    let cam_view = buffer_layout.add_uniforms(1);
    let mut cam_buffer = device.new_buffer(&mut buffer_layout, MemoryType::CpuToGpu, TransferType::None);
    //text setup
    let mut buffer_layout = device.new_buffer_layout();
    let index_view = buffer_layout.add_indices(indices.len());
    let vertex_view = buffer_layout.add_attributes(vertices.len());
    let mut buffer = device.new_buffer(&mut buffer_layout, MemoryType::CpuToGpu, TransferType::None);
    //buffer filling
    {
        let mut map = buffer.map();
        map.write_indices(&index_view, 0, &indices);
        map.write_attributes(&vertex_view, 0, &vertices);
    }
    //texture creation and setup
    /*
    let img = image::io::Reader::open("data/gras.png").unwrap().decode().unwrap().to_bgra();
    let (width, height) = img.dimensions();
    let image_type = ImageType { channel: ImageChannelType::BgraSrgb, width, height, layers: None };
    let texture = device.new_image(image_type, ImageUsage::Texture { mipmapping: true });
    let mut stage_buffer = device.new_image_buffer(image_type);
    stage_buffer.write(&img);
    graphic_command_pool.new_command_buffer().copy_image(graphic_queue, &stage_buffer, &texture, 0, device.new_fence(false)).mark.wait();
    */
    let sampler = device.new_sampler(&SamplerInfo
    {
        min_filter: SamplerFilter::Linear,
        mag_filter: SamplerFilter::Linear,
        mipmap_filter: SamplerFilter::Linear,
        address_mode: SamplerAddressMode::ClampToEdge
    });
    //descriptor setup
    let descriptor_layout = device.new_descriptor_set_layout(0, vec!
    [
        DescriptorBindingInfo::from_struct::<CamBinding>(1, true, false), //binding 0
        DescriptorBindingInfo::from_sampler(ImageChannelType::RUnorm, 1, false, true) //binding 1
    ]);
    let mut descriptor = device.new_descriptor_sets(vec![(&descriptor_layout, 1)]).remove(0).remove(0);
    descriptor.update_struct(0, &cam_buffer, &cam_view);
    descriptor.update_sampler(1, &[&atlas_image], &sampler);
    //swapchain stuff
    let msaa = Msaa::X4;
    let swapchain_stuff_generator = |old_swapchain: Option<SwapchainStuff>, width: u32, height: u32|
    {
        //swapchain creation
        let swapchain = device.new_swapchain(old_swapchain.map(|stuff| stuff.swapchain), (width, height), true);
        //image buffers
        let color_buffer = device.new_image(ImageType { channel: Swapchain::IMAGE_CHANNEL_TYPE, width, height, layers: None }, ImageUsage::Attachment { depth: false, samples: msaa, texture: false, transfer_src: false });
        let depth_buffer = device.new_image(ImageType { channel: ImageChannelType::DSfloat, width, height, layers: None }, ImageUsage::Attachment { depth: true, samples: msaa, texture: false, transfer_src: false });
        //renderpass & pipeline creation
        let render_pass = device.new_render_pass
        (
            &[&RenderPassColorAttachment::Image
            {
                image_channel_type: Swapchain::IMAGE_CHANNEL_TYPE,
                samples: msaa,
                load: ColorAttachmentLoad::Clear { color: [0.0, 0.0, 0.0, 1.0] },
                store: AttachmentStore::Store,
                initial_layout: ImageLayout::Undefined,
                final_layout: ImageLayout::Attachment
            },
            &RenderPassColorAttachment::Swapchain(SwapchainLoad::DontCare)],
            Some(&RenderPassDepthAttachment
            {
                image_channel_type: ImageChannelType::DSfloat,
                samples: msaa,
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
        let pipeline_layout = device.new_pipeline_layout(&[&descriptor_layout]);
        let mut pipeline_info = PipelineInfo::new(width, height);
        pipeline_info.samples = msaa;
        pipeline_info.blend = true;
        let pipeline = device.new_pipeline
        (
            &render_pass,
            0,
            VERTEX,
            FRAGMENT,
            &[&AttributeGroupInfo::from::<Char>()],
            &pipeline_layout,
            &pipeline_info
        );
        //command buffer creation and filling
        let mut command_buffers = swapchain.new_objects(&mut |_| graphic_command_pool.new_command_buffer());
        for (command_buffer, framebuffer) in command_buffers.iter_mut().zip(framebuffers.iter())
        {
            let mut record = command_buffer.record();
            record
                .render_pass(&render_pass, &framebuffer)
                .bind_descriptor_sets(&pipeline_layout, &[&descriptor])
                .bind_pipeline(&pipeline)
                .bind_attributes(&[&AttributeBinding::from(&buffer, &vertex_view)])
                .draw(&DrawMode::Index(IndexBinding::from(&buffer, &index_view), index_count), 1);
        }
        SwapchainStuff { swapchain, color_buffer, depth_buffer, render_pass, framebuffers, pipeline, command_buffers }
    };
    let (width, height) = window.inner_size().into();
    let swapchain_stuff = std::cell::RefCell::new(Some(swapchain_stuff_generator(None, width, height)));
    let mut cam = Camera::new();
    cam.build_projection(width as f32 / height as f32);
    //synchronization elements
    let image_available = swapchain_stuff.borrow().as_ref().unwrap().swapchain.new_cycle(&mut || device.new_semaphore());
    let rendering_finished = swapchain_stuff.borrow().as_ref().unwrap().swapchain.new_cycle(&mut || device.new_semaphore());
    let may_begin_drawing = swapchain_stuff.borrow().as_ref().unwrap().swapchain.new_cycle(&mut || device.new_fence(true));
    //game loop
    let mut fps = FPS::new(None);
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
                                        None =>
                                        {
                                            let monitor = window.available_monitors().nth(0).unwrap();
                                            let mode = monitor.video_modes().nth(0).unwrap();
                                            window.set_fullscreen(Some(window::Fullscreen::Exclusive(mode)));
                                        },
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
                        if button == MouseButton::Right
                        {
                            cam.rotating = state == ElementState::Pressed
                        }
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
                {
                    let mut map = cam_buffer.map();
                    map.write_uniforms(&cam_view, 0, &[CamBinding { mat: cam.mat().into() }]);
                }
                //render
                let image_available = image_available.get(&swapchain_stuff.borrow().as_ref().unwrap().swapchain);
                let rendering_finished = rendering_finished.get(&swapchain_stuff.borrow().as_ref().unwrap().swapchain);
                let may_begin_drawing = may_begin_drawing.get(&swapchain_stuff.borrow().as_ref().unwrap().swapchain);
                let maybe_image_index = swapchain_stuff.borrow().as_ref().unwrap().swapchain.acquire_next_image(Some(&image_available), None);
                if let Ok(image_index) = maybe_image_index
                {
                    may_begin_drawing.wait();
                    may_begin_drawing.reset();
                    swapchain_stuff.borrow().as_ref().unwrap().command_buffers.get(&image_index).submit(&graphic_queue, &image_available, &rendering_finished, &may_begin_drawing);
                    swapchain_stuff.borrow().as_ref().unwrap().swapchain.present(image_index, &graphic_queue, &rendering_finished);
                } else
                {
                    device.idle();
                    let (width, height) = window.inner_size().into();
                    let old_stuff = swapchain_stuff.replace(None);
                    let new_stuff = swapchain_stuff_generator(old_stuff, width, height);
                    swapchain_stuff.replace(Some(new_stuff));
                    cam.build_projection(width as f32 / height as f32);
                }
            },
            _ => {}
        }
    });
    //wait for shutdown
    device.idle();
}

#[allow(unused)]
struct SwapchainStuff<'a>
{
    swapchain: Swapchain,
    color_buffer: Image,
    depth_buffer: Image,
    render_pass: RenderPass,
    framebuffers: SwapchainObjects<Framebuffer>,
    pipeline: Pipeline,
    command_buffers: SwapchainObjects<CommandBuffer<'a>>
}

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
    fn new() -> Self
    {
        Camera
        {
            proj: Mat4::identity(),
            theta: 0.0,
            phi: 0.0,
            pos: (0.0, 0.0, -10.0),
            forward: false,
            backward: false,
            left: false,
            right: false,
            up: false,
            down: false,
            rotating: false
        }
    }

    fn build_projection(&mut self, aspect: f32)
    {
        self.proj = Mat4::perspective_vulkan(aspect, std::f32::consts::FRAC_PI_8, 0.1, 100.0);
    }

    fn walk(&mut self, dt: f32)
    {
        fn front(cam: &Camera) -> (f32, f32, f32) { (cam.theta.cos() * cam.phi.sin(), -cam.theta.sin(), cam.theta.cos() * cam.phi.cos()) }
        fn left(cam: &Camera) -> (f32, f32, f32) { (-cam.phi.cos(), 0.0, cam.phi.sin()) }
        fn up(_cam: &Camera) -> (f32, f32, f32) { (0.0, -1.0, 0.0) }
        fn walk(cam: &mut Camera, dir: (f32, f32, f32), sign: f32, dt: f32)
        {
            let speed = 5.0 * sign * dt;
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

    fn mat(&self) -> Mat4
    {
        let rot = Mat4::rotation_x(-self.theta) * Mat4::rotation_y(-self.phi);
        let trans = Mat4::translation(-self.pos.0, -self.pos.1, -self.pos.2);
        self.proj * rot * trans
    }
}
