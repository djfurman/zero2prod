use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::email_client::EmailClient;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Panic if we cannot read configuration
    let configuration = get_configuration().expect("Failed to read configuration");

    let db_pool = PgPool::connect_with(configuration.database.with_db())
        .await
        .expect("Failed to connect to Postgres database");

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");

    let email_client = EmailClient::new(
        configuration.email_client.authorization_token,
        configuration.email_client.base_url,
        sender_email,
    );

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );

    let listener = TcpListener::bind(address)?;

    let _ = run(listener, db_pool, email_client)?.await;

    Ok(())
}
