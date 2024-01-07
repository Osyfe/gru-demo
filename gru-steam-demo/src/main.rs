mod ui_utils;
mod data;
mod steam_utils;
mod lobby;
mod net;
mod game;

use gru_opengl::{Context, App, event as raw_event, gl, ui as ui_binding};
use gru_ui::{self as ui, math::Vec2, event};

struct Game
{
    ui: ui::Ui<'static, data::Data, ui_utils::EventTag>,
    binding: ui_binding::Binding,
    data: data::Data,
}

impl App for Game
{
    type Init = steam_utils::SteamInit;

    fn init(ctx: &mut Context, steam: Self::Init) -> Self
    {
        ctx.set_title("gru_steam_demo: Rock Paper Scissor");
        ctx.set_window_dims((800, 450));
        let font = ui::text::Font::new(include_bytes!("../res/Latinia.ttf"));
        let ui = ui::Ui::new(font, ui_utils::build());
        let binding = ui_binding::Binding::new(ctx.gl());
        let data = data::Data::new(steam);
        Self { ui, binding, data }
    }

    fn input(&mut self, ctx: &mut Context, event: raw_event::Event)
    {
        let (width, height) = ctx.window_dims();
        self.binding.event(Vec2(width as f32, height as f32), &event);
    }

    fn frame(&mut self, ctx: &mut Context, _: f32) -> bool
    {
        let (width, height) = ctx.window_dims();
        let size = Vec2(width as f32, height as f32);

        let ui::Frame { paint, events, request, .. } = self.ui.frame(ui::UiConfig { size, scale: 1.0, display_scale_factor: 1.0 }, &mut self.data, self.binding.events().into_iter());
        for event in events { self.data.ui_event(ctx, event); }
        self.data.steam_events();
        self.data.frame(request);

        let gl = ctx.gl();
        self.binding.frame(size, gl, paint);
        let mut rp = gl.render_pass(gl::RenderTarget::Screen, gl::RenderPassInfo { clear_color: Some((0.5, 0.5, 0.5)), clear_depth: false });
        self.binding.render(&mut rp);

        !matches!(self.data.state, data::State::End)
    }

    fn deinit(&mut self, _: &mut Context)
    {
    }
}

fn main()
{
    steam_utils::run(|client| gru_opengl::start::<Game>(client));
}
