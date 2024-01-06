mod steam_utils;
mod lobby;

use gru_opengl::{Context, App, event as raw_event, gl, ui as ui_binding};
use gru_ui::{self as ui, math::Vec2, event};

enum State
{
    Menu,
    Lobby(lobby::LobbyData),
    Match,
    End,
}

struct Data
{
    state: State,
    steam: steam_utils::SteamInit,
}

#[derive(Clone, Copy)]
enum EventTag
{
    CreateLobby,
    LeaveLobby,
}

struct Game
{
    ui: ui::Ui<'static, Data, EventTag>,
    binding: ui_binding::Binding,
    data: Data,
}

impl App for Game
{
    type Init = steam_utils::SteamInit;

    fn init(ctx: &mut Context, steam: Self::Init) -> Self
    {
        ctx.set_title("gru_steam_demo: Rock Paper Scissor");
        ctx.set_window_dims((1600, 900));
        let font = ui::text::Font::new(include_bytes!("../res/Latinia.ttf"));
        let ui = ui::Ui::new(font, ui());
        let binding = ui_binding::Binding::new(ctx.gl());
        let data = Data
        {
            state: State::Menu,
            steam,
        };
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

        let ui::Frame { paint, events, .. } = self.ui.frame(ui::UiConfig { size, scale: 1.0, display_scale_factor: 1.0 }, &mut self.data, self.binding.events().into_iter());
        for event in events { self.data.ui_event(ctx, event); }
        self.data.steam_events();

        let gl = ctx.gl();
        self.binding.frame(size, gl, paint);
        let mut rp = gl.render_pass(gl::RenderTarget::Screen, gl::RenderPassInfo { clear_color: Some((0.5, 0.5, 0.5)), clear_depth: false });
        self.binding.render(&mut rp);

        !matches!(self.data.state, State::End)
    }

    fn deinit(&mut self, _: &mut Context)
    {
    }
}

impl Data
{
    fn ui_event(&mut self, ctx: &mut Context, event: &mut event::Event<EventTag>)
    {
        match event
        {
            event::Event::Hardware(event::EventPod { used: false, event }) => match event
            {
                event::HardwareEvent::Key { key, pressed: true } => match key
                {
                    event::Key::Escape => self.state = State::End,
                    event::Key::F => ctx.set_fullscreen(!ctx.fullscreen()),
                    _ => {},
                },
                _ => {},
            },
            event::Event::Logic(event::LogicEvent::Clicked(tag, event::MouseButton::Primary)) => match tag
            {
                EventTag::CreateLobby =>
                {
                    if let Some(lobby) = lobby::LobbyData::new(&self.steam.client)
                    {
                        self.state = State::Lobby(lobby);
                    }
                },
                EventTag::LeaveLobby =>
                {
                    if let State::Lobby(lobby) = &mut self.state { lobby.leave(); }
                    else { unreachable!(); }
                    self.state = State::Menu;
                },
            },
            _ => {},
        }
    }

    fn steam_events(&mut self)
    {
        use steam_utils::SteamEvent as Event;
        for event in self.steam.events.try_iter()
        {
            match event
            {
                Event::JoinLobby(id) =>
                {
                    if let Some(lobby) = lobby::LobbyData::join(&self.steam.client, id)
                    {
                        self.state = State::Lobby(lobby);
                    }
                }
            }
        }
    }
}

fn ui() -> impl ui::Widget<Data, EventTag>
{
    use ui::widget::{WidgetExt, layout::*, primitive::*, compose::*};
    use EventTag::*;

    let menu = Flex::column()
        .with(Label::new().size(2.0).own("Menu").align().center_h())
        .with(Empty.fix().height(1.0))
        .with(Label::new().own("Create Lobby").pad().horizontal(0.5).align().bg().response().event(CreateLobby))
        .align()
        .pad().horizontal(1.0).vertical(1.0);

    let lobby = Flex::column()
        .with(Label::new().size(2.0).own("Lobby").align().center_h())
        .with(Empty.fix().height(1.0))
        .with(Label::new().own("Leave Lobby").pad().horizontal(0.5).align().bg().response().event(LeaveLobby))
        .align()
        .pad().horizontal(1.0).vertical(1.0);

    let set = Set::new()
        .with(menu.maybe(|data: &mut Data| matches!(data.state, State::Menu)))
        .with(lobby.maybe(|data: &mut Data| matches!(data.state, State::Lobby(_))));

    Flex::column()
        .with(Label::new().size(3.0).own("Rock Paper Scissors").align().center_h())
        .with(Empty.fix().height(2.0))
        .with(set)
        .pad().vertical(1.0)
}

fn main()
{
    steam_utils::run(|client| gru_opengl::start::<Game>(client));
}
