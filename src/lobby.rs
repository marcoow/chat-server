use crate::messages::{Connect, Disconnect, UserMessage, WsMessage};
use actix::prelude::{Actor, Context, Handler, Recipient};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

type Socket = Recipient<WsMessage>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Event {
    SelfJoined(Uuid),
    UserPresent(Uuid),
    UserJoined(Uuid),
    UserLeft(Uuid),
    ICECandidate(Uuid, String),
    RTCConnectionOffer(Uuid, String),
    RTCConnectionAnswer(Uuid, String),
}

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
                let event = Event::UserJoined(msg.self_id);
                let json = serde_json::to_string_pretty(&event).unwrap();
                self.send_message(&json, conn_id)
            });

        // send me all the uuids of everyone who is already there
        self.sessions
            .keys()
            .filter(|conn_id| *conn_id.to_owned() != msg.self_id)
            .for_each(|conn_id| {
                let event = Event::UserPresent(*conn_id);
                let json = serde_json::to_string_pretty(&event).unwrap();
                self.send_message(&json, &msg.self_id)
            });

        let event = Event::SelfJoined(msg.self_id);
        let json = serde_json::to_string_pretty(&event).unwrap();
        self.send_message(&json, &msg.self_id);
    }
}

impl Handler<Disconnect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) -> Self::Result {
        // remove the address
        self.sessions.remove(&msg.id);

        // send to everyone in the room that new uuid just left
        self.sessions.keys().for_each(|conn_id| {
            let event = Event::UserLeft(msg.id);
            let json = serde_json::to_string_pretty(&event).unwrap();
            self.send_message(&json, conn_id);
        });
    }
}

impl Handler<UserMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: UserMessage, _: &mut Context<Self>) -> Self::Result {
        match msg.event {
            Event::ICECandidate(id, description) => {
                let event = Event::ICECandidate(msg.self_id, description);
                let json = serde_json::to_string_pretty(&event).unwrap();
                self.send_message(&json, &id)
            }
            Event::RTCConnectionOffer(id, description) => {
                let event = Event::RTCConnectionOffer(msg.self_id, description);
                let json = serde_json::to_string_pretty(&event).unwrap();
                self.send_message(&json, &id)
            }
            Event::RTCConnectionAnswer(id, description) => {
                let event = Event::RTCConnectionAnswer(msg.self_id, description);
                let json = serde_json::to_string_pretty(&event).unwrap();
                self.send_message(&json, &id)
            }
            event => println!("unknown event: {:?}", event),
        }
    }
}
