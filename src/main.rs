use actix::{Actor, Addr};
use actix_web::{
    get, middleware::Logger, web, web::Data, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use actix_web_actors::ws;

mod messages;

mod lobby;
use lobby::Lobby;

mod connection;
use connection::Connection;

#[get("/{lobby_id}")]
pub async fn start_connection(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<Lobby>>,
) -> Result<HttpResponse, Error> {
    let ws = Connection::new(srv.get_ref().clone());

    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let lobby = Data::new(Lobby::new().start());

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let bind_to = "127.0.0.1:4000";
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(lobby.clone())
            .service(start_connection)
    })
    .bind(bind_to)?
    .run();

    println!("Server running on {}", bind_to);

    server.await
}
