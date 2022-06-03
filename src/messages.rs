use actix::Recipient;
use actix_derive::Message;
use uuid::Uuid;

pub enum ClientKind {
    Admin,
    User(String),
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct WebSocketMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientConnect {
    pub addr: Recipient<WebSocketMessage>,
    pub kind: ClientKind,
    pub id: Uuid,
}

impl ClientConnect {
    pub fn user(addr: Recipient<WebSocketMessage>, id: Uuid, name: String) -> ClientConnect {
        ClientConnect {
            id,
            addr,
            kind: ClientKind::User(name),
        }
    }

    pub fn admin(addr: Recipient<WebSocketMessage>, id: Uuid) -> ClientConnect {
        ClientConnect {
            id,
            addr,
            kind: ClientKind::Admin,
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientDisconnect {
    pub id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub id: Uuid,
    pub payload: String,
}
