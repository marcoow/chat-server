use crate::lobby::Event;
use actix::Recipient;
use actix_derive::Message;
use uuid::Uuid;

//WsConn responds to this to pipe it through to the actual client
#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

//WsConn sends this to the lobby to say "put me in please"
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<WsMessage>,
    pub id: Uuid,
}

//WsConn sends this to a lobby to say "take me out please"
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UserMessage {
    pub id: Uuid,
    pub event: Event,
}
