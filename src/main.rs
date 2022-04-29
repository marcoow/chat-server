use actix::{Actor, Addr};
use actix_web::{
    get, middleware::Logger, web, web::Data, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use actix_web_actors::ws;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::{uuid, Uuid};

mod messages;

mod lobby;
use lobby::Lobby;

mod connection;
use connection::Connection;

pub struct AppState {
    lobbies: Mutex<HashMap<Uuid, Addr<Lobby>>>,
}

impl AppState {
    fn new() -> AppState {
        AppState {
            lobbies: Mutex::new(HashMap::new()),
        }
    }
}

#[get("/{lobby_id}")]
pub async fn start_connection(
    req: HttpRequest,
    stream: web::Payload,
    data: Data<AppState>,
) -> Result<HttpResponse, Error> {
    // TODO: this should be the lobby_id in the path later on
    let lobby_id: Uuid = uuid!("27a64ebc-06c9-4f14-bf8b-fafce92d6396");
    let mut lobbies = data.lobbies.lock().unwrap();
    let lobby_addr = match lobbies.get(&lobby_id) {
        Some(lobby) => lobby.clone(),
        None => {
            let new_lobby = Lobby::new().start();
            lobbies.insert(lobby_id, new_lobby.clone());
            new_lobby
        }
    };
    let ws = Connection::new(lobby_addr);

    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data = Data::new(AppState::new());

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let bind_to = "127.0.0.1:4000";
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(data.clone())
            .service(start_connection)
    })
    .bind(bind_to)?
    .run();

    println!("Server running on {}", bind_to);

    server.await
}
