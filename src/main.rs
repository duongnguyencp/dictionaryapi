use actix_web::{App, HttpServer};
mod models;
mod routes;

#[actix_web::main]

async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(routes::dictionary::search))
        .bind(("127.0.0.1", 5174))?
        .run()
        .await
}
