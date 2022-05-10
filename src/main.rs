use actix_cors::Cors;
use actix_web::{middleware::Logger, web, web::Data, App, HttpServer};

mod app_state;
mod connection;
mod handlers;
mod messages;
mod room;

use app_state::AppState;
use handlers::create_room::create_room;
use handlers::start_connection::start_connection;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data = Data::new(AppState::new());

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let bind_to = "127.0.0.1:4000";
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .app_data(data.clone())
            .service(start_connection)
            .route("/rooms", web::post().to(create_room))
    })
    .bind(bind_to)?
    .run();

    println!("Server running on {}", bind_to);

    server.await
}
