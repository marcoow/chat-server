crate::messages::{ClientActorMessage, Connect, Disconnect, WsMessage};
use actix::prelude::{Actor, Context, Handler, Recipient};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

Socket = Recipient<WsMessage>;

pub struct Room {
    sessions: HashMap<Uuid, Socket>,          //self id to self
    users: HashSet<Uuid>,
}

impl Default for Room {
    fn default() -> Room {
        Lobby {
            sessions: HashMap::new(),
            users: HashSet::new(),
        }
    }
}

impl Room {
    fn send_message(&self, message: &str, id_to: &Uuid) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient
                .do_send(WsMessage(message.to_owned()));
        } else {
            println!("attempting to send message but couldn't find user id.");
        }
    }
}

impl Actor for Room {
    type Context = Context<Self>;
}
