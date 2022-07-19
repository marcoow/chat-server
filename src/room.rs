use actix::prelude::{Actor, Context, Handler, Recipient};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::iter::repeat_with;
use std::string::String;
use std::time::Duration;
use uuid::Uuid;

use crate::messages::{
    ClientConnect, ClientDisconnect, ClientKind, ClientMessage, WebSocketMessage,
};

const MATCH_DURATION: Duration = Duration::from_secs(120);

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum Event {
    #[serde(rename = "self-joined")]
    SelfJoined { id: Uuid },
    #[serde(rename = "user-joined")]
    UserJoined { id: Uuid, name: String },
    #[serde(rename = "user-present")]
    UserPresent { id: Uuid, name: String },
    #[serde(rename = "user-matched")]
    UserMatched {
        id: Uuid,
        name: String,
        duration: u64,
    },
    #[serde(rename = "user-left")]
    UserLeft { id: Uuid },
    #[serde(rename = "ice-candidate")]
    ICECandidate { id: Uuid, description: String },
    #[serde(rename = "rtc-connection-offer")]
    RTCConnectionOffer { id: Uuid, description: String },
    #[serde(rename = "rtc-connection-answer")]
    RTCConnectionAnswer { id: Uuid, description: String },
    #[serde(rename = "active-matches-changed")]
    ActiveMatchesChanged { matches: Vec<(Uuid, Uuid)> },
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            Event::SelfJoined { id } => write!(f, "SelfJoined ( id: {:?} )", id),
            Event::UserJoined { id, name } => {
                write!(f, "UserJoined ( id: {:?}, name: {:?} )", id, name)
            }
            Event::UserPresent { id, name } => {
                write!(f, "UserPresent ( id: {:?}, name: {:?} )", id, name)
            }
            Event::UserMatched { id, name, duration } => {
                write!(
                    f,
                    "UserMatched ( id: {:?}, name: {:?}, duration: {:?} )",
                    id, name, duration
                )
            }
            Event::UserLeft { id } => write!(f, "UserLeft ( id: {:?} )", id),
            Event::ICECandidate { id, .. } => {
                write!(f, r#"ICECandidate ( id: {:?}, description: "..." )"#, id)
            }
            Event::RTCConnectionOffer { id, .. } => write!(
                f,
                r#"RTCConnectionOffer ( id: {:?}, description: "..." )"#,
                id
            ),
            Event::RTCConnectionAnswer { id, .. } => write!(
                f,
                r#"RTCConnectionAnswer ( id: {:?}, description: "..." )"#,
                id
            ),
            Event::ActiveMatchesChanged { matches } => {
                write!(f, "ActiveMatchesChanged ( matches: {:?} )", matches)
            }
        }
    }
}

struct UserConnectionInfo {
    name: String,
    socket_recipient: Recipient<WebSocketMessage>,
}

struct AdminConnectionInfo {
    socket_recipient: Recipient<WebSocketMessage>,
}

pub struct Room {
    pub id: Uuid,
    pub name: String,
    pub admin_token: String,
    admins: HashMap<Uuid, AdminConnectionInfo>,
    users: HashMap<Uuid, UserConnectionInfo>,
    active_matches: Vec<(Uuid, Uuid)>,
    previous_matches: Vec<(Uuid, Uuid)>,
}

impl Room {
    pub fn new(name: String) -> Room {
        let random_string = repeat_with(fastrand::alphanumeric).take(32).collect();

        Room {
            id: Uuid::new_v4(),
            name,
            admin_token: random_string,
            admins: HashMap::new(),
            users: HashMap::new(),
            active_matches: Vec::new(),
            previous_matches: Vec::new(),
        }
    }

    fn send_message(&self, message: &str, recipient_id: &Uuid) {
        let do_send = |socket_recipient: &Recipient<WebSocketMessage>| {
            let websocket_body = WebSocketMessage(message.to_owned());
            socket_recipient.do_send(websocket_body);
        };

        if let Some(connection_info) = self.users.get(recipient_id) {
            do_send(&connection_info.socket_recipient);
        } else if let Some(connection_info) = self.admins.get(recipient_id) {
            do_send(&connection_info.socket_recipient);
        } else {
            println!(
                "❌ Attempted to send message but couldn't find user id {:?}!",
                recipient_id
            );
        }
    }

    fn send_event(&self, event: Event, recipient_id: &Uuid) {
        println!("✉️ Sending {:?} to {:?}.", event, recipient_id);

        let json = serde_json::to_string_pretty(&event).unwrap();
        self.send_message(&json, recipient_id);
    }

    fn log_current_stats(&self) {
        println!(
            "
ℹ️ Current server stats:\n
        name: {:?}
        admins: {:?} ({:?})
        users: {:?} ({:?})
        active matches: {:?}
        previous matches: {:?}
        ",
            self.name,
            self.admins.keys().len(),
            self.admins.keys(),
            self.users.keys().len(),
            self.users.keys(),
            self.active_matches,
            self.previous_matches
        );
    }
}

impl Actor for Room {
    type Context = Context<Self>;
}

impl Handler<ClientConnect> for Room {
    type Result = ();

    fn handle(&mut self, msg: ClientConnect, _: &mut Context<Self>) -> Self::Result {
        match msg.kind {
            ClientKind::Admin => {
                // store the new admin
                self.admins.insert(
                    msg.id,
                    AdminConnectionInfo {
                        socket_recipient: msg.addr,
                    },
                );

                // send to the admin all already present users
                for (id, info) in self.users.iter() {
                    self.send_event(
                        Event::UserPresent {
                            id: *id,
                            name: info.name.clone(),
                        },
                        &msg.id,
                    );
                }

                // send to all admins in the room the currently active matches
                self.admins.keys().for_each(|conn_id| {
                    self.send_event(
                        Event::ActiveMatchesChanged {
                            matches: self.active_matches.clone(),
                        },
                        conn_id,
                    );
                });
            }
            ClientKind::User(name) => {
                // store the new user
                self.users.insert(
                    msg.id,
                    UserConnectionInfo {
                        name: name.clone(),
                        socket_recipient: msg.addr,
                    },
                );

                // send the user their own ID
                self.send_event(Event::SelfJoined { id: msg.id }, &msg.id);

                // send to all admins in the room that the user joined
                self.admins.keys().for_each(|conn_id| {
                    self.send_event(
                        Event::UserJoined {
                            id: msg.id,
                            name: name.clone(),
                        },
                        conn_id,
                    );
                });

                match self.make_match(msg.id) {
                    Some((self_id, other_user_id)) => {
                        if let Some(other_user) = self.users.get(&other_user_id) {
                            // send the new user the Id of the other user to connect to
                            self.send_event(
                                Event::UserMatched {
                                    id: other_user_id,
                                    name: other_user.name.clone(),
                                    duration: MATCH_DURATION.as_secs(),
                                },
                                &self_id,
                            );

                            // send to all admins in the room the currently active matches
                            self.admins.keys().for_each(|conn_id| {
                                self.send_event(
                                    Event::ActiveMatchesChanged {
                                        matches: self.active_matches.clone(),
                                    },
                                    conn_id,
                                );
                            });
                        } else {
                            // this should probably throw or something
                        }
                    }
                    None => (),
                }
            }
        }

        self.log_current_stats();
    }
}

impl Handler<ClientDisconnect> for Room {
    type Result = ();

    fn handle(&mut self, msg: ClientDisconnect, _: &mut Context<Self>) -> Self::Result {
        // try selecting the client from all user users
        if let Some(_) = self.users.remove(&msg.id) {
            for (a, b) in &self.active_matches {
                // send the other users in the user's active matches that their partner left
                if a == &msg.id {
                    self.send_event(Event::UserLeft { id: msg.id }, b);
                } else if b == &msg.id {
                    self.send_event(Event::UserLeft { id: msg.id }, a);
                }
            }
            self.active_matches
                .retain(|(a, b)| a != &msg.id && b != &msg.id);
            // send to all admins in the room the currently active matches
            self.admins.keys().for_each(|conn_id| {
                self.send_event(
                    Event::ActiveMatchesChanged {
                        matches: self.active_matches.clone(),
                    },
                    conn_id,
                );
            });

            // send to all admins in the room that the user left
            self.admins.keys().for_each(|conn_id| {
                self.send_event(Event::UserLeft { id: msg.id }, conn_id);
            });
        } else {
            // if the client wasn't among user users, it must have been an admin
            // remove the admin without notifying anyone
            self.admins.remove(&msg.id);
        }

        self.log_current_stats();
    }
}

impl Handler<ClientMessage> for Room {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) -> Self::Result {
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
            Ok(event) => println!("⚠️ Unexpected event: {:?}", event),
            Err(_error) => println!("⚠️ Unknown message: {:?}", msg.payload),
        }

        self.log_current_stats();
    }
}

impl Room {
    fn make_match(&mut self, new_user_id: Uuid) -> Option<(Uuid, Uuid)> {
        let next_match = calculate_next_match::<UserConnectionInfo>(
            &new_user_id,
            &self.users,
            &self.previous_matches,
        );

        match next_match {
            None => return None,
            Some(next_match) => {
                self.active_matches.push(next_match);
                self.previous_matches.push(next_match);
                Some(next_match)
            }
        }
    }
}

fn calculate_next_match<T>(
    id: &Uuid,
    ids_list: &HashMap<Uuid, T>,
    exclude_list: &Vec<(Uuid, Uuid)>,
) -> Option<(Uuid, Uuid)> {
    let next_match_id = ids_list
        .keys()
        // only consider ids that are not the id to match
        .filter(|_id| *_id != id)
        // filter ids that are in the exlude_list
        .filter(|_id| {
            !&exclude_list
                .iter()
                .any(|(a, b)| (&a == _id && &b == &id) || (&a == &id && &b == _id))
        })
        .next();

    match next_match_id {
        None => return None,
        Some(next_match_id) => Some((*id, *next_match_id)),
    }
}

#[cfg(test)]
mod tests {
    use super::calculate_next_match;
    use std::collections::HashMap;
    use uuid::{uuid, Uuid};

    #[test]
    fn it_makes_matches_correctly() {
        let mut users = HashMap::<Uuid, ()>::new();
        let exclude_list = Vec::<(Uuid, Uuid)>::new();

        const USER1_ID: Uuid = uuid!("11111111-06c9-4f14-bf8b-fafce92d6396");
        const USER2_ID: Uuid = uuid!("22222222-06c9-4f14-bf8b-fafce92d6396");
        const USER3_ID: Uuid = uuid!("33333333-06c9-4f14-bf8b-fafce92d6396");
        users.insert(USER1_ID, ());
        users.insert(USER2_ID, ());

        // Sorting isn't stable so it either matches user 3 with user 2 first and then user 1 or with user 1 first and then user 2. After those 2 matches it must return None as there are no options left.
        // Running this 100 times to make sure both cases are covered
        let mut i = 1;
        while i <= 100 {
            match calculate_next_match(&USER3_ID, &users, &exclude_list) {
                Some((USER3_ID, USER2_ID)) => {
                    let next_match =
                        calculate_next_match(&USER3_ID, &users, &vec![(USER3_ID, USER2_ID)]);

                    assert_eq!(next_match, Some((USER3_ID, USER1_ID)));

                    let next_match = calculate_next_match(
                        &USER3_ID,
                        &users,
                        &vec![(USER3_ID, USER2_ID), (USER3_ID, USER1_ID)],
                    );

                    assert_eq!(next_match, None);
                }
                Some((USER3_ID, USER1_ID)) => {
                    let next_next_match =
                        calculate_next_match(&USER3_ID, &users, &vec![(USER3_ID, USER1_ID)]);

                    assert_eq!(next_next_match, Some((USER3_ID, USER2_ID)));

                    let next_match = calculate_next_match(
                        &USER3_ID,
                        &users,
                        &vec![(USER3_ID, USER1_ID), (USER3_ID, USER2_ID)],
                    );

                    assert_eq!(next_match, None);
                }
                Some((_, _)) => {
                    panic!();
                }
                None => {
                    panic!();
                }
            }

            i += 1;
        }
    }
}
