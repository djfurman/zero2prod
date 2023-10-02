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
