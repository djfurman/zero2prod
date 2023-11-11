use crate::routes::health_check;
use crate::routes::subscribe;
use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // Wrap the pool using web::Data which is an Arc smart pointer
    let db_pool = web::Data::new(db_pool);

    // Define the server with the correct listener
    let server = HttpServer::new(move || {
        App::new()
            // All middlewares are added with the wrap command
            .wrap(TracingLogger::default())
            .route("/health-check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            // Register the connection as part of the application state
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
