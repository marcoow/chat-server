use crate::messages::{Connect, Disconnect, WsMessage};
use actix::prelude::{Actor, Context, Handler, Recipient};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
type Socket = Recipient<WsMessage>;

pub struct Lobby {
    sessions: HashMap<Uuid, Socket>, //self id to self
}

impl Default for Lobby {
    fn default() -> Lobby {
        Lobby {
            sessions: HashMap::new(),
        }
    }
}

impl Lobby {
    fn send_message(&self, message: &str, id_to: &Uuid) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient.do_send(WsMessage(message.to_owned()));
        } else {
            println!("attempting to send message but couldn't find user id.");
        }
    }
}

impl Actor for Lobby {
    type Context = Context<Self>;
}

impl Handler<Connect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // store the address
        self.sessions.insert(msg.self_id, msg.addr);

        // send to everyone in the room that new uuid just joined
        self.sessions
            .keys()
            .filter(|conn_id| *conn_id.to_owned() != msg.self_id)
            .for_each(|conn_id| {
                self.send_message(&format!("joined: {}", msg.self_id), conn_id)
            });

        // send me all the uuids of everyone who is already there
        self.sessions
            .keys()
            .filter(|conn_id| *conn_id.to_owned() != msg.self_id)
            .for_each(|conn_id| {
                self.send_message(&format!("present: {}", conn_id), &msg.self_id)
            });

        // send self your new uuid
        self.send_message(&format!("your id is {}", msg.self_id), &msg.self_id);
    }
}
