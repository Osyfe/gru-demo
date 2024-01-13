use super::*;
use ui_utils::EventTag;
use gru_ui::lens::Lens;

pub enum State
{
    Menu,
    Lobby(net::LobbyNetworking, lobby::LobbyData),
    Match(net::Networking, game::Match),
    End,
}

#[derive(Lens)]
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

    pub fn ui_event(&mut self, ctx: &mut Context, request: &mut ui::Request, event: &mut event::Event<EventTag>)
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
                        let net = net::LobbyNetworking::new(&self.steam.client);
                        self.state = State::Lobby(net, lobby);
                    }
                },
                EventTag::LeaveLobby =>
                {
                    self.state = State::Menu;
                },
                EventTag::StartMatch =>
                {
                    if let State::Lobby(net, lobby) = &mut self.state
                    {
                        if lobby.members.len() == 2
                        {
                            let networking = net::Networking::new(net, lobby.members[0].0, lobby.members[1].0);
                            networking.send(steam_utils::SteamMessage::Start(lobby.members[0].0, lobby.members[0].1.clone()));
                            let game = game::Match::new(game::TARGET_SCORE, lobby.members[0].1.clone(), lobby.members[1].1.clone());
                            self.state = State::Match(networking, game);
                        } else 
                        { 
                            println!("You need 2 players to play!");
                        }
                    } else { unreachable!("Start Match while not in Lobby State!"); }
                }
                EventTag::Pick(symbol) => 
                {
                    if let State::Match(networking, game) = &mut self.state
                    {
                        networking.send(steam_utils::SteamMessage::Pick(*symbol));
                        if game.current_round.your_turn(*symbol)
                        {
                            request.layout();
                            if let Some(victor) = game.next_round()
                            {
                                println!("Victor: {victor:?}");
                                self.state = State::Menu;
                            }
                        }
                    } else { unreachable!("Pick Symbol while not in Match State!"); }
                },
                EventTag::Abandon =>
                {
                    if let State::Match(_, _) = &mut self.state
                    {
                        self.state = State::Menu;
                    } else { unreachable!("Abandon while not in Match State!"); }
                },
                EventTag::EndApp => self.state = State::End,
            },
            _ => {},
        }
    }

    pub fn steam_events(&mut self, request: &mut ui::Request)
    {
        use steam_utils::{SteamEvent as Event, SteamMessage as Message};
        for event in self.steam.events.try_iter()
        {
            match event
            {
                Event::JoinLobby(id) =>
                {
                    if let Some(lobby) = lobby::LobbyData::join(&self.steam.client, Some(id))
                    {
                        let net = net::LobbyNetworking::new(&self.steam.client);
                        self.state = State::Lobby(net, lobby);
                    } else
                    {
                        println!("Faild to join Lobby {id:?}");
                    }
                },
                Event::Msg(msg) =>
                {
                    match &mut self.state
                    {
                        State::Menu => unreachable!("Received Message while in Menu"),
                        State::Lobby(net, lobby) => match msg
                        {
                            Message::Hi => {},
                            Message::Start(opp_id, opp_name) =>
                            {
                                let your_id = lobby.members[0].0;
                                let your_name = lobby.members[0].1.clone();
                                let networking = net::Networking::new(net, your_id, opp_id);
                                let game = game::Match::new(game::TARGET_SCORE, your_name, opp_name);
                                self.state = State::Match(networking, game);
                            },
                            Message::Pick(_) => unreachable!("Received Symbol while in Lobby"),
                            Message::Abandon => unreachable!("Received Abandon while in Lobby"),
                        },
                        State::Match(_, game) => match msg
                        {
                            Message::Hi => {},
                            Message::Start(_, _) => unreachable!("Received Start Match while in Match"),
                            Message::Pick(symbol) =>
                            {
                                if game.current_round.opp_turn(symbol)
                                {
                                    request.layout();
                                    if let Some(victor) = game.next_round()
                                    {
                                        println!("Victor: {victor:?}");
                                        self.state = State::Menu;
                                    }
                                }
                            },
                            Message::Abandon =>
                            {
                                println!("{} gave up!", game.players.1);
                                self.state = State::Menu;
                            }
                        },
                        State::End => {},
                    }
                },
            }
        }
    }

    pub fn frame(&mut self, request: &mut ui::Request)
    {
        match &mut self.state
        {
            State::Lobby(net, data) =>
            {
                self.steam.msgs_lobby(net);
                data.frame(request);
                net.frame(&data.members);
            },
            State::Match(networking, _) => self.steam.msgs_game(networking),
            _ => {},
        }
    }
}
