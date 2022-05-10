use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct RoomData {
    pub attributes: RoomInfo,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RoomInfo {
    pub name: String,
}

pub async fn create_room(room_data: web::Json<RoomData>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(room_data))
}
