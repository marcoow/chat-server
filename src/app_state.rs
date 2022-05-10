use actix::Addr;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::room::Room;

pub struct AppState {
    pub rooms: Mutex<HashMap<Uuid, Addr<Room>>>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            rooms: Mutex::new(HashMap::new()),
        }
    }
}
