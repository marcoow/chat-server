use actix::{Actor, Addr};
use actix_web::{
    get, middleware::Logger, web, web::Data, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use actix_web_actors::ws;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::{uuid, Uuid};

mod messages;

mod room;
use room::Room;

mod connection;
use connection::Connection;

pub struct AppState {
    rooms: Mutex<HashMap<Uuid, Addr<Room>>>,
}

impl AppState {
    fn new() -> AppState {
        AppState {
            rooms: Mutex::new(HashMap::new()),
        }
    }
}

#[get("/{room_id}")]
pub async fn start_connection(
    req: HttpRequest,
    stream: web::Payload,
    data: Data<AppState>,
) -> Result<HttpResponse, Error> {
    // TODO: this should be the room_id in the path later on
    let room_id: Uuid = uuid!("27a64ebc-06c9-4f14-bf8b-fafce92d6396");
    let mut rooms = data.rooms.lock().unwrap();
    let room_addr = match rooms.get(&room_id) {
        Some(room) => room.clone(),
        None => {
            let new_room = Room::new("test".to_string()).start();
            rooms.insert(room_id, new_room.clone());
            new_room
        }
    };
    let ws = Connection::new(room_addr);

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
