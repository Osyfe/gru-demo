use steamworks as steam;

type Manager = steam::ClientManager;

pub struct LobbyData
{
    matchmaking: steam::Matchmaking<Manager>,
    lobby: Option<steam::LobbyId>,
}

impl LobbyData
{
    pub fn new(client: &steam::Client) -> Self
    {
        let matchmaking = client.matchmaking();
        matchmaking.create_lobby(steam::LobbyType::FriendsOnly, 2, |_| println!("lobby created"));
        let lobby = None;
        Self { matchmaking, lobby }
    }

    pub fn leave(&mut self)
    {
        if let Some(id) = self.lobby { self.matchmaking.leave_lobby(id) }
    }
}
