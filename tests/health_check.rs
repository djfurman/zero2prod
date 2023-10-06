use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

pub struct TestApp {
    pub api_address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    // Setup a random free ephemeral port for testing purposes
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    // Pull the port number from the listener
    let port = listener.local_addr().unwrap().port();
    // Construct the API's address
    let api_address = format!("http://127.0.0.1:{}", port);
    // Pull the configuration for the database
    let configuration = get_configuration().expect("Failed to read configuration.");
    // Setup the connection to the database pool
    let db_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    // Start the testing server
    let server = run(listener, db_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp {
        api_address,
        db_pool,
    }
}

#[tokio::test]
async fn health_check_works() {
    // Arrange
    // Pull back the application data used for the test server
    let app = spawn_app().await;
    // Needs the `reqwest` crate to run HTTP requests against the server
    // Define a client for HTTP requests in testing
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health-check", &app.api_address))
        .send()
        .await
        .expect("Failed to run request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
