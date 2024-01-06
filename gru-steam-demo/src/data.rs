use super::*;
use ui_utils::EventTag;

pub enum State
{
    Menu,
    Lobby(lobby::LobbyData),
    Match,
    End,
}

pub struct Data
{
    pub state: State,
    pub steam: steam_utils::SteamInit,
}

impl Data
{
    pub fn new(steam: steam_utils::SteamInit) -> Self
    {
        Self
        {
            state: State::Menu,
            steam,
        }
    }

    pub fn ui_event(&mut self, ctx: &mut Context, event: &mut event::Event<EventTag>)
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
                    if let Some(lobby) = lobby::LobbyData::join(&self.steam.client, None)
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
                EventTag::StartMatch =>
                {
                    if let State::Lobby(lobby) = &mut self.state
                    {
                        if lobby.members.len() == 2
                        {
                            todo!();
                        } else { println!("You need 2 players to play!"); }
                    } else { unreachable!(); }
                }
            },
            _ => {},
        }
    }

    pub fn steam_events(&mut self)
    {
        use steam_utils::SteamEvent as Event;
        for event in self.steam.events.try_iter()
        {
            match event
            {
                Event::JoinLobby(id) =>
                {
                    if let Some(lobby) = lobby::LobbyData::join(&self.steam.client, Some(id))
                    {
                        self.state = State::Lobby(lobby);
                    }
                }
            }
        }
    }

    pub fn frame(&mut self, request: &mut ui::Request)
    {
        match &mut self.state
        {
            State::Lobby(data) => data.frame(request),
            _ => {},
        }
    }
}
