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
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let body = "name=Daniel%20Furman&email=djfurman%40users.noreply.github.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.api_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    // Ensure we have the proper status code
    assert_eq!(200, response.status().as_u16());

    // Verify the database
    let saved_record = sqlx::query!("Select email, name From subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    // Ensure we have the correct database record
    assert_eq!(saved_record.email, "djfurman@users.noreply.github.com");
    assert_eq!(saved_record.name, "Daniel Furman");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=daniel%20furman", "missing email input"),
        (
            "email=djfurman%40users.noreply.github.com",
            "missing name input",
        ),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app.api_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not faile with 400 Bad Request when the payload was {}.",
            error_message
        )
    }
}
