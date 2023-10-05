use std::net::TcpListener;
use zero2prod::run;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    // Pull back the address used for the test server
    let api_address = spawn_app();
    // Needs the `reqwest` crate to run HTTP requests against the server
    // Define a client for HTTP requests in testing
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health-check", &api_address))
        .send()
        .await
        .expect("Failed to run request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let api_address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let body = "name=Daniel%20Furman&email=djfurman%40users.noreply.github.com";
    let response = client
        .post(&format!("{}/subscriptions", &api_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let api_address = spawn_app();
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
            .post(&format!("{}/subscriptions", &api_address))
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

fn spawn_app() -> String {
    // Setup a random free ephemeral port for testing purposes
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    // Pull the port number from the listener
    let port = listener.local_addr().unwrap().port();
    // Start the testing server
    let server = zero2prod::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
