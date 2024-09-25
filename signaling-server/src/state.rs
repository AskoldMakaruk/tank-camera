use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};

use futures_channel::mpsc::UnboundedSender;
use log::*;
use protocol::{SignalEnum, TankId, UserId};
use scc::HashMap;
use tokio_tungstenite::tungstenite::Message;

type Tx = UnboundedSender<Message>;
pub type PeerMap = Arc<HashMap<SocketAddr, Tx>>;
pub type UserList = Arc<HashMap<UserId, SocketAddr>>;
pub type TankList = Arc<HashMap<TankId, SocketAddr>>;

pub type SessionList = Arc<HashMap<TankId, Option<UserId>>>;

static PEERS: OnceLock<PeerMap> = OnceLock::new();
static USERS: OnceLock<UserList> = OnceLock::new();
static TANKS: OnceLock<TankList> = OnceLock::new();
static SESSIONS: OnceLock<SessionList> = OnceLock::new();

fn peers<'a>() -> &'a PeerMap {
    PEERS.get_or_init(|| PeerMap::default())
}

fn users<'a>() -> &'a UserList {
    USERS.get_or_init(|| UserList::default())
}

fn tanks<'a>() -> &'a TankList {
    TANKS.get_or_init(|| TankList::default())
}

fn sessions<'a>() -> &'a SessionList {
    SESSIONS.get_or_init(|| SessionList::default())
}

pub fn insert_peer(addr: SocketAddr, tx: Tx) {
    peers().insert(addr, tx.clone());
}

pub fn remove_peer(addr: &SocketAddr) {
    peers().remove(addr);
}

pub fn insert_user(addr: SocketAddr, user_id: UserId) {
    users().insert(user_id, addr);
}

pub fn remove_user(addr: &UserId) {
    users().remove(addr);
}

pub fn insert_tank(addr: SocketAddr, tank_id: TankId) {
    tanks().insert(tank_id, addr);
}

pub fn remove_tank(addr: &TankId) {
    tanks().remove(addr);
}

pub fn get_tank_list() -> Vec<TankId> {
    let mut result = vec![];
    tanks().scan(|k, _| {
        result.push(k.to_owned());
    });

    result
}

pub fn send_message_to_tank(tank_id: &TankId, message: SignalEnum) -> anyhow::Result<()> {
    if let Some(entry) = tanks().get(tank_id) {
        let addr: &SocketAddr = &entry;
        send(addr, message)?;
    };
    Ok(())
}

pub fn send_message_to_operator(operator: &UserId, message: SignalEnum) -> anyhow::Result<()> {
    if let Some(entry) = users().get(operator) {
        let addr: &SocketAddr = &entry;
        send(addr, message)?;
    }
    Ok(())
}

fn send(addr: &SocketAddr, message: SignalEnum) -> Result<(), anyhow::Error> {
    let sender = match peers().get(addr) {
        Some(x) => x,
        None => {
            warn!("Peer was connection dropped from Hashmap, do nothing");
            return Err(anyhow::Error::msg(
                "Peer was connection dropped from Hashmap, do nothing",
            ));
        }
    };
    let message = serde_json::to_string(&message).unwrap();
    debug!("Sending {} to {}", message, addr);
    sender.unbounded_send(Message::Text(message))?;

    Ok(())
}
