use steamworks::{self as steam, networking_messages as net, networking_types as net_types};
use super::steam_utils::{self, Serde};
use std::time::{Instant, Duration};

const CHANNEL: u32 = 72894;
type Manager = steam::ClientManager;

pub struct LobbyNetworking
{
    net: Option<net::NetworkingMessages<Manager>>,
    last_hi: Instant,
}

impl LobbyNetworking
{
    pub fn frame(&mut self, members: &[(steam::SteamId, String)])
    {
        let now = Instant::now();
        if now - self.last_hi > Duration::from_secs(1)
        {
            for (id, _) in &members[1..]
            {
                self.send(*id, steam_utils::SteamMessage::Hi);
            }
            self.last_hi = now;
        }
    }

    pub fn send(&self, id: steam::SteamId, msg: steam_utils::SteamMessage)
    {
        if let Some(net) = &self.net
        {
            send(net, id, msg);
        }
    }

    pub fn recv(&self) -> Option<steam_utils::SteamMessage>
    {
        if let Some(net) = &self.net
        {
            recv(net)
        } else { None }
    }
}

impl LobbyNetworking
{
    pub fn new(client: &steam::Client) -> Self
    {
        let net = client.networking_messages();
        net.receive_messages_on_channel(CHANNEL, 1000);
        let last_hi = Instant::now() - Duration::from_secs(10);
        Self { net: Some(net), last_hi }
    }
}

pub struct Networking
{
    net: net::NetworkingMessages<Manager>,
    _your_id: steam::SteamId,
    opp_id: steam::SteamId,
}

impl Networking
{
    pub fn new(net: &mut LobbyNetworking, your_id: steam::SteamId, opp_id: steam::SteamId) -> Self
    {
        let net = net.net.take().expect("Starting Match more than once");
        Self { net, _your_id: your_id, opp_id }
    }

    pub fn send(&self, msg: steam_utils::SteamMessage) { send(&self.net, self.opp_id, msg); }
    pub fn recv(&self) -> Option<steam_utils::SteamMessage> { recv(&self.net) }
}

impl Drop for Networking
{
    fn drop(&mut self)
    {
        self.send(steam_utils::SteamMessage::Abandon);
    }
}

fn send(net: &net::NetworkingMessages<Manager>, id: steam::SteamId, msg: steam_utils::SteamMessage)
{
    println!("sending: {msg:?}");
    let mut buf = Vec::with_capacity(std::mem::size_of::<steam_utils::SteamMessage>());
    msg.to_bytes(&mut buf);
    net.send_message_to_user(net_types::NetworkingIdentity::new_steam_id(id), net_types::SendFlags::RELIABLE, &buf, CHANNEL).unwrap();
}

fn recv(net: &net::NetworkingMessages<Manager>) -> Option<steam_utils::SteamMessage>
{
    let msgs = net.receive_messages_on_channel(CHANNEL, 1);
    let msg = if msgs.len() > 0 { steam_utils::SteamMessage::from_bytes(msgs[0].data()) } else { None };
    if let Some(msg) = &msg { println!("received: {msg:?}"); }
    msg
}
