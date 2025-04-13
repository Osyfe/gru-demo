mod demo_ui;
mod render;
mod warrior;
mod camera;
mod light;

use gru_wgpu::{App, wgpu, ui::{self, event, lens::Lens}, ui_render::ui_config};
use gru_misc::futures;

pub fn start()
{
    gru_wgpu::run::<Demo>(());
}

enum Render
{
    Loading(futures::State<'static, (warrior::Warrior, wgpu::BindGroupLayout)>),
    Done(render::Render)
}

#[derive(Lens)]
pub struct Demo
{
    depth_buffer: Option<(wgpu::Texture, wgpu::TextureView)>,
    render: Render,
    cam: camera::Cam,
    light: light::Light,
    ui: bool
}

impl App for Demo
{
    const BACKENDS: wgpu::Backends = wgpu::Backends::union(wgpu::Backends::DX12, wgpu::Backends::GL);
    const DEPTH_FORMAT: Option<wgpu::TextureFormat> = Some(render::DEPTH_FORMAT);
    type Init = ();
    type UiEvent = demo_ui::UiEvent;

    fn ui() -> ui::Ui<'static, Self, Self::UiEvent> { demo_ui::build() }

    fn init(_: Self::Init, ctx: &mut gru_wgpu::Context<Self>) -> Self
    {
        ctx.window.set_title("gru-wgpu-demo");
        let depth_buffer = None;
        let render = Render::Loading(futures::State::from(warrior::Warrior::load(ctx.graphics.device.clone(), ctx.graphics.queue.clone())));
        let cam = camera::Cam::new();
        let light = light::Light::new();
        let ui = false;
        Self { depth_buffer, render, cam, light, ui }
    }

    fn frame(&mut self, ctx: &mut gru_wgpu::Context<Self>, dt: f32) -> bool
    {
        let ui_frame = ctx.ui.frame(ui_config(&ctx.window, 1.0), self, ctx.input.events().iter());
        ctx.ui_render.update(&ctx.graphics, &ui_frame.paint);

        for event in ui_frame.events.iter()
        {
            match event
            {
                event::Event::Hardware(event::EventPod { event: event::HardwareEvent::CloseWindow, used: false }) => return true,
                _ => {}
            }
        }
        self.cam.input(ui_frame.events);

        drop(ui_frame);

        let render = match &mut self.render
        {
            Render::Loading(loader) =>
            {
                if let futures::Query::Done((warrior, texture_bind_group_layout)) = loader.query()
                {
                    let render = render::Render::new(&ctx.graphics.device, ctx.graphics.view_format(), texture_bind_group_layout, warrior);
                    self.render = Render::Done(render);
                }
                None
            },
            Render::Done(render) => Some(render)
        };
        
        if let Some((surface_texture, surface_view)) = ctx.graphics.current_surface().unwrap()
        {
            //depth buffer
            let dims = ctx.window.inner_size().into();
            if match &self.depth_buffer
            {
                Some((buffer, _)) => (buffer.width(), buffer.height()) != dims,
                None => true
            }
            {
                let texture_descr = wgpu::TextureDescriptor
                {
                    label: None,
                    size: wgpu::Extent3d { width: dims.0, height: dims.1, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: render::DEPTH_FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[]
                };
                let texture = ctx.graphics.device.create_texture(&texture_descr);
                let texture_view_descr = wgpu::TextureViewDescriptor
                {
                    label: None,
                    format: None,
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    usage: Some(wgpu::TextureUsages::empty()),
                    aspect: wgpu::TextureAspect::DepthOnly,
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: 0,
                    array_layer_count: None
                };
                let texture_view = texture.create_view(&texture_view_descr);
                self.depth_buffer = Some((texture, texture_view));
            }
            let depth_view = &self.depth_buffer.as_ref().unwrap().1;

            //render
            let mut encoder = ctx.graphics.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            let render_pass_descr =  wgpu::RenderPassDescriptor
            {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment
                {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations
                    {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.7, g: 0.9, b: 0.8, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment
                {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Discard }),
                    stencil_ops: None
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            };
            let mut rp = encoder.begin_render_pass(&render_pass_descr);

            //warrior
            if let Some(render) = render
            {
                let dims = ctx.window.inner_size().into();
                let (cam_pos, cam_mat) = self.cam.build(dims);
                let uniform_v = render::UniformVertex { cam: cam_mat };
                let uniform_f = self.light.build(dt, cam_pos);
                render.render(&ctx.graphics.queue, &mut rp, uniform_v, uniform_f);
            }
            //ui
            ctx.ui_render.render(&mut rp);

            //finish
            drop(rp);
            let command_buf = encoder.finish();
            ctx.graphics.queue.submit([command_buf]);
            surface_texture.present();
        }

        false
    }
}
