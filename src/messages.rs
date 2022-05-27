use actix::Recipient;
use actix_derive::Message;
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub struct WebSocketMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct UserConnect {
    pub addr: Recipient<WebSocketMessage>,
    pub id: Uuid,
    pub name: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct AdminConnect {
    pub addr: Recipient<WebSocketMessage>,
    pub id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UserDisconnect {
    pub id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct AdminDisconnect {
    pub id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UserMessage {
    pub id: Uuid,
    pub payload: String,
}
