use steamworks as steam;
use std::sync::mpsc;
use super::{game, net};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum SteamMessage
{
    Start(steam::SteamId, String),
    Pick(game::Symbol),
    Abandon,
}

pub enum SteamEvent
{
    JoinLobby(steam::LobbyId),
    Msg(SteamMessage),
}

pub struct SteamInit
{
    pub client: steam::Client,
    pub events: mpsc::Receiver<SteamEvent>,
    send: mpsc::SyncSender<SteamEvent>,
}

pub fn run<F: FnOnce(SteamInit)>(f: F)
{
    let (client, single) = steam::Client::init().expect("Probably Steam not running!");
    client.utils().set_overlay_notification_position(steam::NotificationPosition::BottomRight);

    let (send, recv) = mpsc::sync_channel(1);
    let send_lobby = send.clone();
    let _lobby = client.register_callback(move |event: steam::GameLobbyJoinRequested| send_lobby.send(SteamEvent::JoinLobby(event.lobby_steam_id)).unwrap());
    
    let init = SteamInit { client, events: recv, send };
    let driver = CBDriver::start(single);
    f(init);
    driver.stop();
}

impl SteamInit
{
    pub fn msgs(&mut self, net: &net::Networking)
    {
        if let Some(msg) = net.recv()
        {
            self.send.send(SteamEvent::Msg(msg)).unwrap();
        }
    }
}

struct CBDriver(mpsc::SyncSender<()>);

impl CBDriver
{
    fn start(single: steam::SingleClient) -> Self
    {
        let (send, recv) = mpsc::sync_channel(1);
        std::thread::spawn(move ||
        {
            loop
            {
                single.run_callbacks();
                std::thread::sleep(std::time::Duration::from_millis(16));
                if recv.try_recv().is_ok() { break; }
            }
        });
        Self(send)
    }

    fn stop(self)
    {
        self.0.send(()).unwrap();
    }
}

pub struct BlockOn<T: 'static + Send>(mpsc::SyncSender<T>);

impl<T: 'static + Send> BlockOn<T>
{
    pub fn with<F: FnOnce(BlockOn<T>)>(f: F) -> T
    {
        let (send, recv) = mpsc::sync_channel(1);
        f(Self(send));
        recv.recv().unwrap()
    }

    pub fn cb(self) -> impl FnOnce(T) + 'static + Send
    {
        move |res| self.0.send(res).unwrap()
    }
}

pub trait Serde: Sized
{
	fn to_bytes(&self, buf: &mut Vec<u8>);
	fn from_bytes(data: &[u8]) -> Option<Self>;
}

impl<T: Serialize + for<'de> Deserialize<'de>> Serde for T
{
	fn to_bytes(&self, buf: &mut Vec<u8>) { buf.clear(); bincode::serialize_into(buf, self).unwrap() }
	fn from_bytes(data: &[u8]) -> Option<Self> { result_to_option(bincode::deserialize_from(data)) }
}

#[inline(always)]
fn result_to_option<T, E: std::fmt::Debug>(result: Result<T, E>) -> Option<T>
{
    match result
    {
        Ok(ok) => Some(ok),
        Err(err) =>
        {
            println!("Serde: {:?}", err);
            None
        }
    }
}
