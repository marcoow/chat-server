use actix::prelude::Actor;
use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
