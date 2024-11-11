use gru_wgpu::{App, input::HardwareEvent, wgpu};

pub fn start()
{
    gru_wgpu::run::<Demo>(());
}

pub struct Demo;

impl App for Demo
{
    const BACKENDS: wgpu::Backends = wgpu::Backends::GL;
    type Init = ();

    fn init(_: Self::Init) -> Self
    {
        log::warn!("Log Test");
        Self
    }

    fn frame(&mut self, ctx: &mut gru_wgpu::Context) -> bool
    {
        for event in &ctx.input.events
        {
            match event
            {
                HardwareEvent::CloseWindow => return true,
                _ => {}
            }
        }

        if let Some((surface_texture, surface_view)) = ctx.graphics.current_surface().unwrap()
        {
            let mut encoder = ctx.graphics.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            let render_pass_descr = wgpu::RenderPassDescriptor
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
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            };

            let rp = encoder.begin_render_pass(&render_pass_descr);
            drop(rp);
            let command_buf = encoder.finish();

            ctx.graphics.queue.submit([command_buf]);
            surface_texture.present();
        }

        false
    }

    fn deinit(self, _: &mut gru_wgpu::Context) -> Option<Self::Init>
    {
        None
    }
}
