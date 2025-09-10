use std::time::Duration;

use actix_web::{App, HttpServer};
use tracing::info;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::fmt;
mod middleware;
mod models;
mod routes;
mod utils;
mod validate;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let format = fmt::format()
        .with_level(false) // don't include levels in formatted output
        .with_target(false) // don't include targets
        .with_thread_ids(true) // include the thread ID of the current thread
        .with_thread_names(true) // include the name of the current thread
        .compact();
    // Init global subscriber
    tracing_subscriber::fmt().event_format(format).init();
    info!("🚀 Starting Dictionary Web server...");
    HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(middleware::cors::init_cors())
            .wrap(middleware::rate_limit::init_rl())
            .wrap(middleware::timeout::TimeoutHandler::new(
                Duration::from_secs(10),
            ))
            .wrap(middleware::helmet::SecurityHeaders)
            .wrap(actix_web::middleware::Compress::default())
            .service(routes::dictionary::search)
    })
    .bind(("127.0.0.1", 5174))?
    .run()
    .await
}
