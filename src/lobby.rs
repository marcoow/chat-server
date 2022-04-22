use actix::prelude::{Actor, Context, Handler, Recipient};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

//type Socket = Recipient<WsMessage>;

pub struct Lobby {
    //sessions: HashMap<Uuid, Socket>, //self id to self
    lobbys: HashMap<Uuid, HashSet<Uuid>>, //lobby id  to list of users id
}

impl Default for Lobby {
    fn default() -> Lobby {
        Lobby {
            //sessions: HashMap::new(),
            lobbys: HashMap::new(),
        }
    }
}

// impl Lobby {
//     fn send_message(&self, message: &str, id_to: &Uuid) {
//         if let Some(socket_recipient) = self.sessions.get(id_to) {
//             let _ = socket_recipient
//                 .do_send(WsMessage(message.to_owned()));
//         } else {
//             println!("attempting to send message but couldn't find user id.");
//         }
//     }
// }

impl Actor for Lobby {
    type Context = Context<Self>;
}
