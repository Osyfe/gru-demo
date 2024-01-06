use super::steam_utils::BlockOn;
use steamworks as steam;

type Manager = steam::ClientManager;

pub struct LobbyData
{
    matchmaking: steam::Matchmaking<Manager>,
    friends: steam::Friends<Manager>,
    lobby: steam::LobbyId,
    member_count: usize,
    pub members: Vec<String>,
}

impl LobbyData
{
    pub fn join(client: &steam::Client, lobby: Option<steam::LobbyId>) -> Option<Self>
    {
        let matchmaking = client.matchmaking();
        let friends = client.friends();
        fn print_error<E: std::fmt::Debug>(error: E) { println!("{error:?}"); }
        match lobby
        {
            None => BlockOn::with(|block| matchmaking.create_lobby(steam::LobbyType::FriendsOnly, 2, block.cb())).map_err(print_error),
            Some(lobby) => BlockOn::with(|block| matchmaking.join_lobby(lobby, block.cb())).map_err(print_error),
        }.ok().map(|lobby| Self { matchmaking, friends, lobby, member_count: 0, members: Vec::new() })
    }

    pub fn leave(&mut self)
    {
        self.matchmaking.leave_lobby(self.lobby)
    }

    pub fn frame(&mut self, request: &mut gru_ui::Request)
    {
        let new_count = self.matchmaking.lobby_member_count(self.lobby);
        if new_count != self.member_count
        {
            let list = self.matchmaking.lobby_members(self.lobby);
            self.members.clear();
            for member in list
            {
                let name = self.friends.get_friend(member).name();
                self.members.push(name);
            }
            self.member_count = new_count;
            request.widget();
        }
    }
}
