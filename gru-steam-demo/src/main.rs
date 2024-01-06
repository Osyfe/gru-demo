mod lobby;

use gru_opengl::{Context, App, event as raw_event, gl, ui as ui_binding};
use gru_ui::{self as ui, math::Vec2, event};
use steamworks as steam;

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
    steam: steam::Client,
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
    steam_callbacks: (steam::SingleClient, Vec<steam::CallbackHandle>),
}

impl App for Game
{
    type Init = (steam::Client, (steam::SingleClient, Vec<steam::CallbackHandle>));

    fn init(ctx: &mut Context, (steam, steam_callbacks): Self::Init) -> Self
    {
        ctx.set_title("gru_steam_demo: Rock Paper Scissor");
        let font = ui::text::Font::new(include_bytes!("../res/Latinia.ttf"));
        let ui = ui::Ui::new(font, ui());
        let binding = ui_binding::Binding::new(ctx.gl());
        let data = Data
        {
            state: State::Menu,
            steam,
        };
        Self { ui, binding, data, steam_callbacks }
    }

    fn input(&mut self, ctx: &mut Context, event: raw_event::Event)
    {
        let (width, height) = ctx.window_dims();
        self.binding.event(Vec2(width as f32, height as f32), &event);
    }

    fn frame(&mut self, ctx: &mut Context, dt: f32) -> bool
    {
        let (width, height) = ctx.window_dims();
        let size = Vec2(width as f32, height as f32);

        let ui::Frame { paint, events, request, .. } = self.ui.frame(ui::UiConfig { size, scale: 1.0, display_scale_factor: 1.0 }, &mut self.data, self.binding.events().into_iter());
        for event in events { self.data.event(ctx, event); }

        self.steam_callbacks.0.run_callbacks();

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
    fn event(&mut self, ctx: &mut Context, event: &mut event::Event<EventTag>)
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
                    let lobby = lobby::LobbyData::new(&self.steam);
                    self.state = State::Lobby(lobby);
                },
                EventTag::LeaveLobby =>
                {
                    if let State::Lobby(lobby) = &mut self.state { lobby.leave(); }
                    else { unreachable!(); }
                },
            },
            _ => {},
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

fn steam_callbacks(steam: &steam::Client) -> Vec<steam::CallbackHandle>
{
    let lobby = steam.register_callback(|event: steam::GameLobbyJoinRequested| println!("lobby id: {:?}, friend id: {:?}", event.lobby_steam_id, event.friend_steam_id));
    vec![lobby]
}

fn main()
{
    let (client, single) = steam::Client::init().unwrap();
    client.utils().set_overlay_notification_position(steam::NotificationPosition::BottomRight);
    let callbacks = steam_callbacks(&client);
    gru_opengl::start::<Game>((client, (single, callbacks)));
}
