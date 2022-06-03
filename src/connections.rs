use actix::{
    fut, Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, ContextFutureSpawner, Handler,
    Running, StreamHandler, WrapFuture,
};
use actix_web_actors::ws;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::messages::{ClientConnect, ClientDisconnect, ClientMessage, WebSocketMessage};
use crate::room::Room;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Connection {
    room_addr: Addr<Room>,
    last_heartbeat: Instant,
    id: Uuid,
    kind: ConnectionKind,
}

enum ConnectionKind {
    Admin,
    User(String),
}

impl Connection {
    pub fn user(name: String, room_addr: Addr<Room>) -> Connection {
        Connection {
            id: Uuid::new_v4(),
            kind: ConnectionKind::User(name),
            room_addr,
            last_heartbeat: Instant::now(),
        }
    }

    pub fn admin(room_addr: Addr<Room>) -> Connection {
        Connection {
            id: Uuid::new_v4(),
            kind: ConnectionKind::Admin,
            room_addr,
            last_heartbeat: Instant::now(),
        }
    }

    fn heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.last_heartbeat) > CLIENT_TIMEOUT {
                println!("Disconnecting because of failed heartbeat");
                act.send_disconnect_message(act.id);
                ctx.stop();
                return;
            }

            ctx.ping(b"PING");
        });
    }

    fn send_disconnect_message(&self, id: Uuid) {
        self.room_addr.do_send(ClientDisconnect { id });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Connection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.last_heartbeat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.last_heartbeat = Instant::now();
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                ctx.stop();
            }
            Ok(ws::Message::Nop) => (),
            Ok(ws::Message::Text(text)) => match self.kind {
                ConnectionKind::User(_) => {
                    self.room_addr.do_send(ClientMessage {
                        id: self.id,
                        payload: text.to_string(),
                    });
                }
                ConnectionKind::Admin => (),
            },
            Err(e) => panic!("Error! {:?}", e),
        }
    }
}

impl Actor for Connection {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);

        let addr = ctx.address();
        let message = match &self.kind {
            ConnectionKind::User(name) => {
                ClientConnect::user(addr.recipient(), self.id, name.clone())
            }
            ConnectionKind::Admin => ClientConnect::admin(addr.recipient(), self.id),
        };

        self.room_addr
            .send(message)
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(_res) => (),
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.send_disconnect_message(self.id);
        Running::Stop
    }
}

impl Handler<WebSocketMessage> for Connection {
    type Result = ();

    fn handle(&mut self, msg: WebSocketMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}
