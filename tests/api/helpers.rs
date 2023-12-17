use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::email_client::EmailClient;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    // Cannot assign the output of `get_subscriber`` to a variable based on the value `TEST_LOG`
    // because the sink is part of the type returned by `get_subscriber``, therefore
    // they are not the same type. Could work around this, but this is the most straight-forward approach

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub api_address: String,
    pub db_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    // Let's get some logging going for the test suite
    Lazy::force(&TRACING);

    // Setup a random free ephemeral port for testing purposes
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    // Pull the port number from the listener
    let port = listener.local_addr().unwrap().port();
    // Construct the API's address
    let api_address = format!("http://127.0.0.1:{}", port);
    // Pull the configuration for the database
    let mut configuration = get_configuration().expect("Failed to read configuration.");
    // Randomize the database using a Uuid V4
    configuration.database.name = Uuid::new_v4().to_string();
    // Setup the connection to the database pool
    let db_pool = configure_database(&configuration.database).await;

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");

    let email_timeout = configuration.email_client.timeout();

    let email_client = EmailClient::new(
        configuration.email_client.authorization_token,
        configuration.email_client.base_url,
        sender_email,
        email_timeout,
    );

    // Start the testing server
    let server =
        run(listener, db_pool.clone(), email_client.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp {
        api_address,
        db_pool,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create
    let mut db_conn = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    db_conn
        .execute(format!(r#"Create Database "{}";"#, config.name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate
    let db_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres new db.");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate the database.");

    db_pool
}
