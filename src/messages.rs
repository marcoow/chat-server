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
    pub self_id: Uuid,
}

//WsConn sends this to a lobby to say "take me out please"
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
}

//client sends this to the lobby for the lobby to echo out.
// #[derive(Message)]
// #[rtype(result = "()")]
// pub struct ClientActorMessage {
//     pub id: Uuid,
//     pub msg: String,
//     pub lobby_id: Uuid
// }
