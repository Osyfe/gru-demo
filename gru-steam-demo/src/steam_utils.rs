use steamworks as steam;
use std::sync::mpsc;

use crate::game::Symbol;

pub enum SteamEvent
{
    JoinLobby(steam::LobbyId),
    Pick(Symbol)
}

pub struct SteamInit
{
    pub client: steam::Client,
    pub events: mpsc::Receiver<SteamEvent>,
}

pub fn run<F: FnOnce(SteamInit)>(f: F)
{
    let (client, single) = steam::Client::init().unwrap();
    client.utils().set_overlay_notification_position(steam::NotificationPosition::BottomRight);

    let (send, recv) = mpsc::sync_channel(1);
    let send_lobby = send.clone();
    let _lobby = client.register_callback(move |event: steam::GameLobbyJoinRequested| send_lobby.send(SteamEvent::JoinLobby(event.lobby_steam_id)).unwrap());
    
    let init = SteamInit { client, events: recv };
    let driver = CBDriver::start(single);
    f(init);
    driver.stop();
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