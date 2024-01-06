use super::steam_utils::BlockOn;
use steamworks as steam;

type Manager = steam::ClientManager;

pub struct LobbyData
{
    matchmaking: steam::Matchmaking<Manager>,
    lobby: steam::LobbyId,
}

impl LobbyData
{
    pub fn new(client: &steam::Client) -> Option<Self>
    {
        let matchmaking = client.matchmaking();
        let res = BlockOn::with(|block| matchmaking.create_lobby(steam::LobbyType::FriendsOnly, 2, block.cb()));
        match res
        {
            Ok(lobby) => Some(Self { matchmaking, lobby }),
            Err(err) =>
            {
                println!("{err:?}");
                None
            }
        }
    }

    pub fn join(client: &steam::Client, lobby: steam::LobbyId) -> Option<Self>
    {
        let matchmaking = client.matchmaking();
        let res = BlockOn::with(|block| matchmaking.join_lobby(lobby, block.cb()));
        match res
        {
            Ok(lobby) => Some(Self { matchmaking, lobby }),
            Err(err) =>
            {
                println!("{err:?}");
                None
            }
        }
    }

    pub fn leave(&mut self)
    {
        self.matchmaking.leave_lobby(self.lobby)
    }
}
