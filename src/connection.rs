use actix::{
    fut, Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, ContextFutureSpawner, Handler,
    Running, StreamHandler, WrapFuture,
};
use actix_web_actors::ws;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::messages::{UserConnect, UserDisconnect, UserMessage, WebSocketMessage};

use crate::room::Room;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Connection {
    room_addr: Addr<Room>,
    last_heartbeat: Instant,
    id: Uuid,
    name: String,
}

impl Connection {
    pub fn new(name: String, room_addr: Addr<Room>) -> Connection {
        Connection {
            id: Uuid::new_v4(),
            name,
            room_addr,
            last_heartbeat: Instant::now(),
        }
    }

    fn heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.last_heartbeat) > CLIENT_TIMEOUT {
                println!("Disconnecting because of failed heartbeat");
                act.room_addr.do_send(UserDisconnect { id: act.id });
                ctx.stop();
                return;
            }

            ctx.ping(b"PING");
        });
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
            Ok(ws::Message::Text(text)) => {
                self.room_addr.do_send(UserMessage {
                    id: self.id,
                    payload: text.to_string(),
                });
            }
            Err(e) => panic!("Error! {:?}", e),
        }
    }
}

impl Actor for Connection {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);

        let addr = ctx.address();
        self.room_addr
            .send(UserConnect {
                addr: addr.recipient(),
                name: self.name.clone(),
                id: self.id,
            })
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
        self.room_addr.do_send(UserDisconnect { id: self.id });
        Running::Stop
    }
}

impl Handler<WebSocketMessage> for Connection {
    type Result = ();

    fn handle(&mut self, msg: WebSocketMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}
