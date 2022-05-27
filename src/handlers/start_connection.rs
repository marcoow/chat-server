use actix::Actor;
use actix_web::{get, web, web::Data, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use uuid::{uuid, Uuid};

use crate::app_state::AppState;
use crate::connection::Connection;
use crate::room::Room;

#[get("/{room_id}/{name}")]
pub async fn start_connection(
    req: HttpRequest,
    stream: web::Payload,
    data: Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
    let (_room_id, name) = path.into_inner();
    // TODO: this should be the room_id in the path later on
    let room_id: Uuid = uuid!("27a64ebc-06c9-4f14-bf8b-fafce92d6396");
    let mut rooms = data.rooms.lock().unwrap();
    let room_addr = match rooms.get(&room_id) {
        Some(room) => room.clone(),
        None => {
            // TODO: this should later respond with 404
            let new_room = Room::new("test".to_string()).start();
            rooms.insert(room_id, new_room.clone());
            new_room
        }
    };
    let ws = Connection::new(name, room_addr);

    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}
