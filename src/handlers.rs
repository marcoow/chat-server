use actix::Actor;
use actix_web::{get, web, web::Data, Error, HttpRequest, HttpResponse, Result};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use uuid::{uuid, Uuid};

use crate::admin_connection::AdminConnection;
use crate::connection::Connection;
use crate::app_state::AppState;
use crate::room::Room;

#[derive(Debug, Deserialize, Serialize)]
pub struct RoomData {
    pub attributes: RoomInfo,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RoomInfo {
    pub name: String,
    pub admin_token: Option<String>,
    pub id: Option<Uuid>,
}

pub async fn create_room(
    room_data: web::Json<RoomData>,
    data: web::Data<AppState>,
) -> Result<HttpResponse> {
    let mut rooms = data.rooms.lock().unwrap();
    let new_room = Room::new(room_data.attributes.name.clone());

    let response = RoomData {
        attributes: RoomInfo {
            id: Some(new_room.id),
            name: new_room.name.clone(),
            admin_token: Some(new_room.admin_token.clone()),
        },
    };

    let new_room_id = new_room.id.clone();
    let new_room_addr = new_room.start();
    rooms.insert(new_room_id, new_room_addr.clone());

    Ok(HttpResponse::Ok().json(response))
}

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
