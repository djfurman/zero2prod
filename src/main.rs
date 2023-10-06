use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Panic if we cannot read configuration
    let configuration = get_configuration().expect("Failed to read configuration");

    let pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres database");

    let address = format!("127.0.0.1:{}", configuration.application_listen_port);

    let listener = TcpListener::bind(address)?;

    run(listener, pool)?.await
}
