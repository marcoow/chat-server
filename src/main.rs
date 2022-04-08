// see https://levelup.gitconnected.com/websockets-in-actix-web-full-tutorial-websockets-actors-f7f9484f5086
use actix::{fut, ActorContext};
use actix::prelude::{Context};
use actix::{Actor, Addr, Running, StreamHandler, WrapFuture, ActorFuture, ContextFutureSpawner};
use actix_web::{middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer, get};
use actix_web_actors::ws;
use actix_web_actors::ws::Message::Text;
use actix::{AsyncContext, Handler};
use std::time::{Duration, Instant};
use uuid::Uuid;

mod messages;
use messages::{WsMessage, Connect, Disconnect, ClientActorMessage};

mod room;
use room::Room;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

struct Connection {
    room: Uuid,
    hb: Instant,
    id: Uuid,
}

impl Connection {
    pub fn new(room: Uuid) -> Connection {
        Connection {
            id: Uuid::new_v4(),
            room,
            hb: Instant::now(),
        }
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Disconnecting failed heartbeat");
                act.lobby_addr.do_send(Disconnect { id: act.id, room_id: act.room });
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
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
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
            Ok(Text(s)) => self.lobby_addr.do_send(ClientActorMessage {
                id: self.id,
                msg: s,
                room_id: self.room
            }),
            Err(e) => panic!("{}", e),
        }
    }
}

impl Actor for Connection {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    
        let addr = ctx.address();
        self.lobby_addr
            .send(Connect {
                addr: addr.recipient(),
                lobby_id: self.room,
                self_id: self.id,
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
        self.lobby_addr.do_send(Disconnect { id: self.id, room_id: self.room });
        Running::Stop
    }
}

impl Handler<WsMessage> for Connection {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}


#[get("/")]
pub async fn start_connection(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<Room>>,
) -> Result<HttpResponse, Error> {
    let ws = Connection::new(
        Uuid::new_v4(),
    );

    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let chat_server = Room::default().start(); //create and spin up a lobby

    HttpServer::new(move || {
        App::new()
            .service(start_connection) //. rename with "as" import or naming conflict
            .data(chat_server.clone()) //register the lobby
    })
    .bind("127.0.0.1:4000")?
    .run()
    .await
}