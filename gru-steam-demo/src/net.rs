use steamworks::{self as steam, networking_messages as net, networking_types as net_types};
use super::steam_utils::{self, Serde};

const CHANNEL: u32 = 72894;
type Manager = steam::ClientManager;

pub struct LobbyNetworking(Option<net::NetworkingMessages<Manager>>);

impl LobbyNetworking
{
    pub fn recv(&self) -> Option<steam_utils::SteamMessage>
    {
        if let Some(net) = &self.0
        {
            let msgs = net.receive_messages_on_channel(CHANNEL, 1);
            let msg = if msgs.len() > 0 { steam_utils::SteamMessage::from_bytes(msgs[0].data()) } else { None };
            if let Some(msg) = &msg { println!("received in lobby: {msg:?}"); }
            msg
        } else { None }
    }
}

impl LobbyNetworking
{
    pub fn new(client: &steam::Client) -> Self
    {
        let net = client.networking_messages();
        Self(Some(net))
    }
}

pub struct Networking
{
    net: net::NetworkingMessages<Manager>,
    your_id: steam::SteamId,
    opp_id: steam::SteamId,
}

impl Networking
{
    pub fn new(net: &mut LobbyNetworking, your_id: steam::SteamId, opp_id: steam::SteamId) -> Self
    {
        let net = net.0.take().expect("Starting Match more than once");
        Self { net, your_id, opp_id }
    }

    pub fn send(&self, msg: steam_utils::SteamMessage)
    {
        println!("sending: {msg:?}");
        let mut buf = Vec::with_capacity(std::mem::size_of::<steam_utils::SteamMessage>());
        msg.to_bytes(&mut buf);
        self.net.send_message_to_user(net_types::NetworkingIdentity::new_steam_id(self.opp_id), net_types::SendFlags::RELIABLE, &buf, CHANNEL).unwrap();
    }

    pub fn recv(&self) -> Option<steam_utils::SteamMessage>
    {
        let msgs = self.net.receive_messages_on_channel(CHANNEL, 1);
        let msg = if msgs.len() > 0 { steam_utils::SteamMessage::from_bytes(msgs[0].data()) } else { None };
        if let Some(msg) = &msg { println!("received: {msg:?}"); }
        msg
    }
}
