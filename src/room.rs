use actix::prelude::{Actor, Context, Handler, Recipient};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::repeat_with;
use std::string::String;
use uuid::Uuid;

use crate::messages::{Connect, Disconnect, UserMessage, WebSocketMessage};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Event {
    #[serde(rename = "self-joined")]
    SelfJoined { id: Uuid },
    #[serde(rename = "user-matched")]
    UserMatched { id: Uuid },
    #[serde(rename = "user-left")]
    UserLeft { id: Uuid },
    #[serde(rename = "ice-candidate")]
    ICECandidate { id: Uuid, description: String },
    #[serde(rename = "rtc-connection-offer")]
    RTCConnectionOffer { id: Uuid, description: String },
    #[serde(rename = "rtc-connection-answer")]
    RTCConnectionAnswer { id: Uuid, description: String },
}

struct ConnectionInfo {
    name: String,
    socket_recipient: Recipient<WebSocketMessage>,
}

pub struct Room {
    pub id: Uuid,
    pub name: String,
    pub admin_token: String,
    sessions: HashMap<Uuid, ConnectionInfo>,
    previous_matches: Vec<(Uuid, Uuid)>,
}

impl Room {
    pub fn new(name: String) -> Room {
        let random_string = repeat_with(fastrand::alphanumeric).take(32).collect();

        Room {
            id: Uuid::new_v4(),
            name,
            admin_token: random_string,
            sessions: HashMap::new(),
            previous_matches: Vec::new(),
        }
    }

    fn send_message(&self, message: &str, recipient_id: &Uuid) {
        if let Some(connection_info) = self.sessions.get(recipient_id) {
            connection_info
                .socket_recipient
                .do_send(WebSocketMessage(message.to_owned()));
        } else {
            println!(
                "attempting to send message but couldn't find user id {:?}.",
                recipient_id
            );
        }
    }

    fn send_event(&self, event: Event, recipient_id: &Uuid) {
        let json = serde_json::to_string_pretty(&event).unwrap();
        self.send_message(&json, recipient_id);
    }
}

impl Actor for Room {
    type Context = Context<Self>;
}

impl Handler<Connect> for Room {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // store the new user
        self.sessions.insert(
            msg.id,
            ConnectionInfo {
                name: msg.name,
                socket_recipient: msg.addr,
            },
        );

        self.send_event(Event::SelfJoined { id: msg.id }, &msg.id);

        match self.make_match(msg.id) {
            Some((self_id, other_user_id)) => {
                // send the new user the Id of the other user to connect to
                self.send_event(Event::UserMatched { id: other_user_id }, &self_id);
            }
            None => (),
        }
    }
}

impl Handler<Disconnect> for Room {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) -> Self::Result {
        // remove the disconnected user
        self.sessions.remove(&msg.id);

        // send to everyone in the room that new uuid just left
        self.sessions.keys().for_each(|conn_id| {
            self.send_event(Event::UserLeft { id: msg.id }, conn_id);
        });
    }
}

impl Handler<UserMessage> for Room {
    type Result = ();

    fn handle(&mut self, msg: UserMessage, _: &mut Context<Self>) -> Self::Result {
        let event: Result<Event, serde_json::Error> = serde_json::from_str(&msg.payload);

        match event {
            Ok(Event::ICECandidate { id, description }) => self.send_event(
                Event::ICECandidate {
                    id: msg.id,
                    description,
                },
                &id,
            ),
            Ok(Event::RTCConnectionOffer { id, description }) => self.send_event(
                Event::RTCConnectionOffer {
                    id: msg.id,
                    description,
                },
                &id,
            ),
            Ok(Event::RTCConnectionAnswer { id, description }) => self.send_event(
                Event::RTCConnectionAnswer {
                    id: msg.id,
                    description,
                },
                &id,
            ),
            Ok(event) => println!("unexpected event: {:?}", event),
            Err(_error) => println!("unknown message: {:?}", msg.payload),
        }
    }
}

impl Room {
    // TODO: this also needs to consider currently active matches and must not match with a user that's currently in an active match
    fn make_match(&mut self, new_user_id: Uuid) -> Option<(Uuid, Uuid)> {
        let new_user = [new_user_id];
        let other_users = self
            .sessions
            .keys()
            .filter(|conn_id| *conn_id.to_owned() != new_user_id);

        let next_match = new_user
            .iter()
            .zip(other_users.clone())
            .map(|(a, b)| (*a, *b))
            .filter(|(a, b)| {
                !self
                    .previous_matches
                    .iter()
                    .any(|(pa, pb)| (pa == a && pb == b) || (pa == b && pb == a))
            })
            .next();

        match next_match {
            None => return None,
            Some(next_match) => {
                self.previous_matches.push(next_match);
                Some(next_match)
            }
        }
    }
}
