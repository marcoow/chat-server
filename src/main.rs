// see https://levelup.gitconnected.com/websockets-in-actix-web-full-tutorial-websockets-actors-f7f9484f5086
use actix::{fut, ActorContext};
use actix::{Actor, Addr, ContextFutureSpawner, Running, StreamHandler, WrapFuture};
use actix::{AsyncContext, Handler};
use actix_web::{get, middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use actix_web_actors::ws::Message::Text;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

mod messages;
use messages::{Connect, Disconnect, WsMessage};

mod lobby;
use lobby::Lobby;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

struct Connection {
    lobby: String,
    lobby_addr: Addr<Lobby>,
    hb: Instant,
    id: Uuid,
}

impl Connection {
    pub fn new(lobby: String, lobby_addr: Addr<Lobby>) -> Connection {
        Connection {
            id: Uuid::new_v4(),
            lobby,
            lobby_addr,
            hb: Instant::now(),
        }
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Disconnecting failed heartbeat");
                //act.lobby_addr.do_send(Disconnect { id: act.id, lobby_id: act.lobby });
                ctx.stop();
                return;
            }

            ctx.ping(b"PING");
        });
    }
}

struct AppState {
    lobby_addrs: Addr<Lobby>,
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
            Ok(Text(s)) => (),
            // Ok(Text(s)) => self.lobby_addr.do_send(ClientActorMessage {
            //     id: self.id,
            //     msg: s,
            //     lobby_id: self.lobby
            // }),
            Err(e) => panic!("{}", e),
        }
    }
}

impl Actor for Connection {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address();
        // self.lobby_addr
        //     .send(Connect {
        //         addr: addr.recipient(),
        //         lobby_id: self.lobby,
        //         self_id: self.id,
        //     })
        //     .into_actor(self)
        //     .then(|res, _, ctx| {
        //         match res {
        //             Ok(_res) => (),
        //             _ => ctx.stop(),
        //         }
        //         fut::ready(())
        //     })
        //     .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        //self.lobby_addr.do_send(Disconnect { id: self.id, lobby_id: self.lobby });
        Running::Stop
    }
}

impl Handler<WsMessage> for Connection {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

#[get("/{lobby_id}")]
pub async fn start_connection(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<(String,)>,
    srv: web::Data<Addr<Lobby>>,
) -> Result<HttpResponse, Error> {
    let lobby_id = path.into_inner().0;
    println!("{}", lobby_id);
    let ws = Connection::new(lobby_id, srv.get_ref().clone());

    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let lobby = Lobby::default().start();

    let bind_to = "127.0.0.1:4000";
    let server = HttpServer::new(move || {
        App::new().data(lobby.clone()).service(start_connection) //. rename with "as" import or naming conflict
    })
    .bind(bind_to)?
    .run();

    println!("Server running on {}", bind_to);
    
    server.await
}
