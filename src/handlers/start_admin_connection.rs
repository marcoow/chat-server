use actix::Actor;
use actix_web::{get, web, web::Data, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use uuid::{uuid, Uuid};

use crate::admin_connection::AdminConnection;
use crate::app_state::AppState;
use crate::room::Room;

#[get("/{room_id}/admin/{admin_token}")]
pub async fn start_admin_connection(
    req: HttpRequest,
    stream: web::Payload,
    data: Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
    let (_room_id, _admin_token) = path.into_inner();
    // TODO: this should be the room_id in the path later on
    //       also, this should check the token is indeed correct
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
    let ws = AdminConnection::new(room_addr);

    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}
